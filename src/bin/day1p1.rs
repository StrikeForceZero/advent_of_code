use std::error::Error;

pub trait SortCopied {
    type Item;
    fn sort_copied(&self) -> Vec<Self::Item>;
}

impl SortCopied for &[u32] {
    type Item = u32;

    fn sort_copied(&self) -> Vec<Self::Item> {
        let mut a = self.to_vec();
        a.sort();
        a
    }
}

fn calc_distance(a: &[u32], b: &[u32]) -> u32 {
    let a = a.sort_copied();
    let b = b.sort_copied();
    let mut distance = 0;
    for (a, b) in a.iter().zip(b.iter()) {
        distance += a.abs_diff(*b);
    }
    distance
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = advent_of_code::read_input_lines(1);
    let mut a = Vec::new();
    let mut b = Vec::new();
    for input_line in lines {
        let input_line = input_line?;
        let input_line = input_line
            .trim()
            .split_whitespace()
            .map(|x| x.parse::<u32>().expect("Invalid input"))
            .collect::<Vec<_>>();
        a.push(input_line[0]);
        b.push(input_line[1]);
    }
    let answer = calc_distance(&a, &b);
    println!("Answer: {answer}");
    Ok(())
}
