use regex::Regex;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let lines = advent_of_code::read_input(3);
    let mul_regex = Regex::new(r"mul\((\d+),(\d+)\)").expect("invalid regex");
    let mut result = 0;
    for input_line in lines {
        let input_line = input_line?;
        let input_line = input_line.trim();
        let matches: Vec<(i32, i32)> = mul_regex
            .captures_iter(input_line)
            .map(|cap| {
                (
                    cap[1].parse::<i32>().expect("failed to parse int"),
                    cap[2].parse::<i32>().expect("failed to parse int"),
                )
            })
            .collect();
        for (a, b) in matches {
            result += a * b;
        }
    }
    let answer = result;
    println!("Answer: {answer}");
    Ok(())
}
