use std::fs::File;
use std::io::{BufRead, BufReader, Lines};

pub fn read_input(day: u32) -> Lines<BufReader<File>> {
    let input_path = format!("inputs/day{}", day);
    let input_file = File::open(input_path).expect("File not found");
    let input_reader = BufReader::new(input_file);
    input_reader.lines().into_iter()
}
