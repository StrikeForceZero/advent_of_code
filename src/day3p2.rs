extern crate core;

use regex::{Captures, Match, Regex};
use std::error::Error;
use std::io::{BufReader, Lines, Read};

struct MatchDetails<T> {
    start: usize,
    end: usize,
    data: T,
}

fn extract_match_details<T>(c: &Captures, f: impl FnOnce(&Captures) -> T) -> MatchDetails<T> {
    let Some(mat) = c.get(0) else { unreachable!() };
    let start = mat.start();
    let end = mat.end();
    let data = f(c);
    MatchDetails { start, end, data }
}

fn parse_lines<T: Read>(lines: Lines<BufReader<T>>) -> Result<i32, Box<dyn Error>> {
    let mul_regex = Regex::new(r"mul\((\d+),(\d+)\)").expect("invalid regex");
    let do_regex = Regex::new(r"do\(\)").expect("invalid regex");
    let dont_regex = Regex::new(r"don't\(\)").expect("invalid regex");

    fn get_last_match<'a>(regex: &Regex, input: &'a str) -> Option<Match<'a>> {
        regex.find_iter(input).last()
    }

    fn has_match<'a>(regex: &Regex, input: &str) -> bool {
        regex.find(input).is_some()
    }

    fn missing_match<'a>(regex: &Regex, input: &str) -> bool {
        !has_match(regex, input)
    }

    let mut result = 0;
    let mut enabled = true;
    for input_line in lines {
        let mut last_index = 0;
        let input_line = input_line?;
        let input_line = input_line.trim();
        let mul_ops: Vec<MatchDetails<(i32, i32)>> = mul_regex
            .captures_iter(input_line)
            .filter_map(|cap| {
                let mat = extract_match_details(&cap, |cap| (
                    cap[1].parse::<i32>().expect("failed to parse int"),
                    cap[2].parse::<i32>().expect("failed to parse int"),
                ));
                // back to back operator check optimization
                if mat.start != last_index + 1 {
                    // not back to back
                    // we need to search the &str slice between the last index and our current match for `do` `don't`
                    let do_dont_search_area = &input_line[last_index..mat.start];
                    let next_enabled = if let Some(preceding_do) = get_last_match(&do_regex, do_dont_search_area) {
                        // preceding_do.end() is relative, we need absolute so add the last_index
                        let next_last_index = last_index + preceding_do.end();
                        debug_assert!(next_last_index > last_index);
                        // shift up the last_index based on our searching
                        last_index = next_last_index;
                        let do_dont_search_area = &input_line[last_index..mat.start];
                        // lastly make sure there are no `don't`s between the `do` and the `mul`
                        missing_match(&dont_regex, do_dont_search_area)
                    } else {
                        // no `do`s found, rely on previous known state, and make sure a `don't` doesn't exist between `mul`s
                        if !enabled {
                            // if enabled is false then don't bother searching for `don't`s
                            false
                        } else {
                            // make sure there are no preceding `don't`s
                            missing_match(&dont_regex, do_dont_search_area)
                        }
                    };
                    if next_enabled != enabled {
                        enabled = next_enabled;
                    }
                } else {
                    // back to back, no need to check for `do()` or `don't()`
                }
                // set the last index to the end of the found `mul`
                last_index = mat.end;
                if enabled {
                    Some(mat)
                } else {
                    None
                }
            })
            .collect();

        for MatchDetails { data: (a, b), .. } in mul_ops {
            result += a * b;
        }
    }
    Ok(result)
}


fn main() -> Result<(), Box<dyn Error>> {
    let lines = advent_of_code::read_input(3);
    let answer = parse_lines(lines)?;
    println!("Answer: {answer}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, Cursor};
    use super::*;
    fn string_to_br_lines(s: &str) -> Lines<BufReader<Cursor<&str>>> {
        let cursor = Cursor::new(s);
        let reader = BufReader::new(cursor);
        reader.lines()
    }

    #[test]
    fn simple_mul() -> Result<(), Box<dyn Error>> {
        assert_eq!(parse_lines(string_to_br_lines("mul(2,3)"))?, 6);
        assert_eq!(parse_lines(string_to_br_lines("mul(2,3)mul(3,4)"))?, 18);
        Ok(())
    }

    #[test]
    fn bad_mul() -> Result<(), Box<dyn Error>> {
        assert_eq!(parse_lines(string_to_br_lines("mu(2,3)"))?, 0);
        assert_eq!(parse_lines(string_to_br_lines("mul()"))?, 0);
        assert_eq!(parse_lines(string_to_br_lines("mul(,)"))?, 0);
        assert_eq!(parse_lines(string_to_br_lines("mul(,1)"))?, 0);
        assert_eq!(parse_lines(string_to_br_lines("mul(1,)"))?, 0);
        assert_eq!(parse_lines(string_to_br_lines("mul(1,1"))?, 0);
        assert_eq!(parse_lines(string_to_br_lines("mul1,1"))?, 0);
        Ok(())
    }

    #[test]
    fn dont_mul() -> Result<(), Box<dyn Error>> {
        assert_eq!(parse_lines(string_to_br_lines("do()don't()mul(2,3)"))?, 0);
        assert_eq!(parse_lines(string_to_br_lines("do()__don't()mul(2,3)"))?, 0);
        Ok(())
    }

    #[test]
    fn do_mul() -> Result<(), Box<dyn Error>> {
        assert_eq!(parse_lines(string_to_br_lines("don't()do()mul(2,3)"))?, 6);
        assert_eq!(parse_lines(string_to_br_lines("don't()__do()mul(2,3)"))?, 6);
        Ok(())
    }

    #[test]
    fn do_mul_dont_mul() -> Result<(), Box<dyn Error>> {
        assert_eq!(parse_lines(string_to_br_lines("don't()do()mul(2,3)don't()mul(1,2)"))?, 6);
        Ok(())
    }

    #[test]
    fn dont_mul_do_mul() -> Result<(), Box<dyn Error>> {
        assert_eq!(parse_lines(string_to_br_lines("don't()mul(1,2)do()mul(2,3)"))?, 6);
        Ok(())
    }

    #[test]
    fn examples() -> Result<(), Box<dyn Error>> {
        // assert_eq!(parse_lines(string_to_br_lines("xmul(2,4)%&mul[3,7]!@^do_not_mul(5,5)+mul(32,64]then(mul(11,8)mul(8,5))"))?, 161);
        assert_eq!(parse_lines(string_to_br_lines("xmul(2,4)&mul[3,7]!^don't()_mul(5,5)+mul(32,64](mul(11,8)undo()?mul(8,5))"))?, 48);
        Ok(())
    }
}
