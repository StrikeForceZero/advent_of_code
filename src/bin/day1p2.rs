use std::collections::HashMap;
use std::error::Error;

fn occurrence_map(items: &[u32]) -> HashMap<u32, u32> {
    let mut map = HashMap::new();
    for &item in items {
        *map.entry(item).or_default() += 1;
    }
    map
}

fn calc_similarity_score(a: &[u32], b: &[u32]) -> u32 {
    let a = occurrence_map(a);
    let b = occurrence_map(b);
    let mut similarity = 0;
    for (&av, &ac) in a.iter() {
        similarity += av * b.get(&av).cloned().unwrap_or_default() * ac;
    }
    similarity
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
    let answer = calc_similarity_score(&a, &b);
    println!("Answer: {answer}");
    Ok(())
}
