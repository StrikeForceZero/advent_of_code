/// incorrect

use std::iter::Rev;
use std::ops::Range;
use std::slice::Iter;
use regex::{Match, Regex};

pub trait CharsToString {
    fn chars_to_string(&self) -> String;
}

impl CharsToString for Vec<char> {
    fn chars_to_string(&self) -> String {
        self.iter().collect()
    }
}

impl CharsToString for Rev<Iter<'_, char>> {
    fn chars_to_string(&self) -> String {
        self.clone().collect()
    }
}

fn check_string(regex: &Regex, s: &Vec<char>) -> usize {
    if s.len() < 4 {
        return 0;
    }
    fn extract_range(m: Match) -> Range<usize> {
        m.start()..m.end()
    }

    let string = s.chars_to_string();
    let matches = regex.find_iter(&string);
    let points = matches.map(extract_range).collect::<Vec<_>>();
    let count = points.len();

    println!("{string}");
    for point in points {
        println!("{:<spaces$}{:><width$}", "", "", spaces=point.start, width=point.end-point.start);
    }

    let reverse_string = &s.iter().rev().chars_to_string();
    let reverse_matches = regex.find_iter(&reverse_string);
    let reverse_points = reverse_matches.map(extract_range).collect::<Vec<_>>();
    let reverse_count = reverse_points.len();

    for point in reverse_points {
        println!("{:<spaces$}{:<<width$}", "", "", spaces=string.len()-point.end, width=point.end-point.start);
    }

    count + reverse_count
}

fn get_main_diagonals<T>(matrix: &Vec<Vec<T>>) -> Vec<Vec<T>>
where
    T: Clone,
{
    let rows = matrix.len();
    if rows == 0 {
        return Vec::new();
    }
    let cols = matrix[0].len();
    let mut diagonals = Vec::new();

    // Top-left to bottom-right diagonals
    for start in 0..(rows + cols - 1) {
        let mut diagonal = Vec::new();
        let mut row = if start < cols { 0 } else { start - cols + 1 };
        let mut col = if start < cols { start } else { cols - 1 };

        while row < rows && col < cols {
            diagonal.push(matrix[row][col].clone());
            row += 1;
            col = col.saturating_sub(1); // Avoids negative index
        }

        diagonals.push(diagonal);
    }

    diagonals
}

fn get_anti_diagonals<T>(matrix: &Vec<Vec<T>>) -> Vec<Vec<T>>
where
    T: Clone,
{
    let rows = matrix.len();
    if rows == 0 {
        return Vec::new();
    }
    let cols = matrix[0].len();
    let mut diagonals = Vec::new();

    // Top-right to bottom-left diagonals
    for start in 0..(rows + cols - 1) {
        let mut diagonal = Vec::new();
        let mut row = if start < cols { 0 } else { start - cols + 1 };
        let mut col = if start < cols { cols - 1 - start } else { 0 };

        while row < rows && col < cols {
            diagonal.push(matrix[row][col].clone());
            row += 1;
            col += 1;
        }

        diagonals.push(diagonal);
    }

    diagonals
}

fn display(matrix: &[Vec<char>]) {
    for row in matrix.iter() {
        // println!("{}", row.iter().collect::<String>());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lines = advent_of_code::read_input(4);
    let mut result = 0;
    let mut data = vec![];
    for input_line in lines {
        let input_line = input_line?;
        let input_line = input_line.trim();
        data.push(input_line.chars().collect::<Vec<_>>());
    }
    let height = data.len();
    let width = data.first().unwrap().len();
    let xmas_regex = Regex::new("XMAS").expect("invalid regex");

    println!("\n--- rows ---");
    let rows = data.clone();
    display(&rows);
    for row in rows {
        result += check_string(&xmas_regex, &row);
    }

    println!("\n--- cols ---");
    let cols = (0..width)
        .map(|x| {
            (0..height)
                .map(|y| data[y][x])
                .collect::<Vec<char>>()
        })
        .collect::<Vec<_>>();
    display(&cols);
    for col in cols {
        result += check_string(&xmas_regex, &col);
    }

    println!("\n--- anti ---");
    let diags = get_main_diagonals(&data);
    display(&diags);
    for diag in diags {
        result += check_string(&xmas_regex, &diag);
    }

    println!("\n--- anti_diags ---");
    let anti_diags = get_anti_diagonals(&data);
    display(&anti_diags);
    for diag in anti_diags {
        result += check_string(&xmas_regex, &diag);
    }

    let answer = result;
    println!("Answer: {answer}"); // should be 18
    Ok(())
}

/*
....XXMAS.
.SAMXMS...
...S..A...
..A.A.MS.X
XMASAMX.MM
X.....XA.A
S.S.S.S.SS
.A.A.A.A.A
..M.M.M.MM
.X.X.XMASX
 */

#[cfg(test)]
mod tests {
    use regex::Regex;

    #[test]
    fn test() {
        assert_eq!(Regex::new("XMAS").expect("invalid regex").find_iter("XMASAMXMAS").count(), 2);
    }
}
