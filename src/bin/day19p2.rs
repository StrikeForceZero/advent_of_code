use advent_of_code::read_input;
use std::collections::HashSet;

fn main() -> anyhow::Result<()> {
    let input = read_input(19)?;
    let (pattern_set, requests) = parse_input(&input)?;
    println!("pattern_set: {pattern_set:?}");
    println!("request count: {}", requests.len());
    println!("finding matches");
    let sum = requests
        .into_iter()
        .map(|request| {
            println!("{}:", request.pattern);
            let matches = pattern_set.matches(&request);
            let len = matches.len();
            println!("# matches: {len}");
            for match_item in matches {
                println!("{match_item:?}");
            }
            len
        })
        .sum::<usize>();
    println!("sum: {sum}");
    Ok(())
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct PatternSet {
    patterns: Vec<String>,
}

impl PatternSet {
    fn can_print(&self, request_pattern: &RequestPattern) -> bool {
        self.matches(request_pattern).len() > 0
    }
    fn matches(&self, request_pattern: &RequestPattern) -> Vec<Vec<String>> {
        // println!("pattern_set: {:?}", self.patterns);
        // println!("request_pattern: {:?}\n", request_pattern.pattern);
        let mut pattern_sets = HashSet::new();
        let total_len = request_pattern.pattern.len();
        let mut branches = self
            .patterns
            .clone()
            .into_iter()
            .map(|pattern| (vec![pattern.len()], pattern))
            .collect::<HashSet<_>>();
        let mut chars = request_pattern.pattern.chars().enumerate().peekable();
        while let Some((ix, char)) = chars.next() {
            println!("searching {char} @ {ix:>3}/{total_len:<3} {}", branches.len());
            let is_last_char = chars.peek().is_none();
            let mut new_branches = HashSet::with_capacity(branches.len());;
            for (splits, left_pattern) in branches.into_iter() {
                // println!("[{ix:>3?}/{total_len:<3}]:[{char}] b: ({:>3}/{:<3}) {} {}", left_pattern.len(), total_len, &left_pattern[0..ix], &left_pattern[ix..]);
                let pattern_char = left_pattern.chars().nth(ix);
                // if left pattern has ran out of matching chars we start adding the next chained match
                if pattern_char.is_none() {
                    for right_pattern in self.patterns.iter() {
                        // find all patterns that match the current character
                        if right_pattern.chars().nth(0) == Some(char) {
                            let mut splits = splits.clone();
                            splits.push(left_pattern.len());
                            let new_pattern = format!("{left_pattern}{right_pattern}");
                            let item = (splits, new_pattern);
                            if is_last_char && right_pattern.len() == 1 {
                                // println!("[is_none] complete: {new_pattern}");
                                // if is_last_char and single char and match then we are done
                                if !pattern_sets.contains(&item) {
                                    pattern_sets.insert(item);
                                }
                            } else if !is_last_char {
                                // println!("[is_none] extended: {new_pattern}");
                                if !new_branches.contains(&item) {
                                    new_branches.insert(item);
                                }
                            } else {
                                // println!("[is_none] discarding: {left_pattern} {right_pattern}");
                            }
                        } else {
                            // println!("[is_none] discarding: {left_pattern} {right_pattern}");
                        }
                    }
                } else if pattern_char == Some(char) {
                    if is_last_char && left_pattern.len() - 1 == ix {
                        println!("complete: {left_pattern}");
                        // if is_last_char and left len - 1 matches ix then we are done
                        let item = (splits, left_pattern);
                        if !pattern_sets.contains(&item) {
                            pattern_sets.insert(item);
                        }
                    } else if !is_last_char {
                        // println!("[is_eq] extended: {left_pattern}");
                        let item = (splits, left_pattern);
                        if !new_branches.contains(&item) {
                            new_branches.insert(item);
                        }
                    } else {
                        // println!("[is_eq]discarding: {left_pattern}");
                    }
                    continue;
                } else {
                    // println!("[is_ne] discarding: {left_pattern}");
                }
            }
            branches = new_branches;
            // println!("\nreset branches: {:?}", branches);
        }
        // println!("\npattern_sets: {:?}", pattern_sets);
        pattern_sets
            .into_iter()
            .map(|(splits, mut pattern)| {
                for split in splits.into_iter().rev() {
                    pattern.insert(split, ' ');
                }
                pattern
                    .split_whitespace()
                    .map(|part| part.to_string())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }
}

fn get_pattern_from_lines<'a>(
    lines: &mut impl Iterator<Item = &'a str>,
) -> anyhow::Result<Vec<String>> {
    let patterns = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("empty input"))?
        .trim()
        .split(", ")
        .map(|part| part.to_string())
        .collect::<Vec<_>>();
    Ok(patterns)
}

impl TryFrom<&str> for PatternSet {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut lines = value.lines();
        let patterns = get_pattern_from_lines(&mut lines)?;
        if lines.next().is_some() {
            anyhow::bail!("unexpected new line");
        }
        Ok(PatternSet { patterns })
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct RequestPattern {
    pattern: String,
}

impl TryFrom<&str> for RequestPattern {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut lines = value.lines();
        let pattern = lines
            .next()
            .ok_or_else(|| anyhow::anyhow!("empty input"))?
            .trim()
            .to_string();
        if lines.next().is_some() {
            anyhow::bail!("unexpected new line");
        }
        Ok(RequestPattern { pattern })
    }
}

fn parse_input(input: &str) -> anyhow::Result<(PatternSet, Vec<RequestPattern>)> {
    let mut lines = input.lines();
    let patterns = lines
        .next()
        .map(PatternSet::try_from)
        .ok_or_else(|| anyhow::anyhow!("empty input"))??;
    if Some(false) == lines.next().map(|line| line.trim().is_empty()) {
        anyhow::bail!("unexpected new line");
    }
    let requests = lines
        .map(RequestPattern::try_from)
        .collect::<Result<Vec<_>, _>>()?;
    Ok((patterns, requests))
}

#[cfg(test)]
mod tests {
    use super::*;
    use advent_of_code::utils::string::deformat_string;

    #[test]
    fn matches_test() -> anyhow::Result<()> {
        let pattern_set = PatternSet::try_from("a, bc")?;
        assert_eq!(
            pattern_set.matches(&RequestPattern::try_from("abc")?),
            vec![vec!["a", "bc"]]
        );
        let pattern_set = PatternSet::try_from("ab, c")?;
        assert_eq!(
            pattern_set.matches(&RequestPattern::try_from("abc")?),
            vec![vec!["ab", "c"]]
        );
        let pattern_set = PatternSet::try_from("abc")?;
        assert_eq!(
            pattern_set.matches(&RequestPattern::try_from("abc")?),
            vec![vec!["abc"]]
        );
        let pattern_set = PatternSet::try_from("a, b, c")?;
        assert_eq!(
            pattern_set.matches(&RequestPattern::try_from("abc")?),
            vec![vec!["a", "b", "c"]]
        );
        Ok(())
    }

    #[test]
    fn examples() -> anyhow::Result<()> {
        let input = deformat_string(
            "
            r, wr, b, g, bwu, rb, gb, br

            brwrr
            bggr
            gbbr
            rrbgbr
            ubwu
            bwurrg
            brgr
            bbrgwb
        ",
        );
        let (pattern_set, requests) = parse_input(&input)?;
        let valid_count = requests
            .iter()
            .map(|request| pattern_set.matches(request).len())
            .sum::<usize>();
        assert_eq!(valid_count, 16);
        Ok(())
    }
}
