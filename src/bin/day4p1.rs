use lazy_static::lazy_static;
use advent_of_code::utils::matrix::{rotate_matrix_180_degrees, rotate_matrix_270_degrees, rotate_matrix_90_degrees, skew_matrix_by};

lazy_static! {
    static ref PATTERN: Vec<Vec<Option<char>>> = {
        vec![
            vec![Some('X'), Some('M'), Some('A'),  Some('S')],
        ]
    };
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lines = advent_of_code::read_input(4);
    let mut data = vec![];
    for input_line in lines {
        let input_line = input_line?;
        let input_line = input_line.trim();
        data.push(input_line.chars().collect::<Vec<_>>());
    }

    let base_patterns = vec![
        PATTERN.clone(), // XMAS
        /*
            X
             M
              A
               S
         */
        skew_matrix_by(&PATTERN, 1),
    ];
    let mut patterns = vec![];
    patterns.extend(base_patterns.clone());
    patterns.extend(base_patterns.iter().map(rotate_matrix_90_degrees).collect::<Vec<_>>());
    patterns.extend(base_patterns.iter().map(rotate_matrix_180_degrees).collect::<Vec<_>>());
    patterns.extend(base_patterns.iter().map(rotate_matrix_270_degrees).collect::<Vec<_>>());

    let rows = data.len();
    let cols = data[0].len();

    let mut count = 0;
    for pattern in patterns {
        let pat_rows = pattern.len();
        let pat_cols = pattern[0].len();
        for row in 0..rows {
            for col in 0..cols {
                if matches_pattern(&data, &pattern, row, col, rows, cols, pat_rows, pat_cols) {
                    count += 1;
                }
            }
        }
    }
    let answer = count;
    println!("Answer: {answer}");
    Ok(())
}

fn matches_pattern(
    grid: &Vec<Vec<char>>,
    pattern: &Vec<Vec<Option<char>>>,
    start_row: usize,
    start_col: usize,
    rows: usize,
    cols: usize,
    pat_rows: usize,
    pat_cols: usize,
) -> bool {
    if start_row + pat_rows > rows || start_col + pat_cols > cols {
        return false; // Out of bounds for the pattern
    }

    for pr in 0..pat_rows {
        for pc in 0..pat_cols {
            let pat_char = pattern[pr][pc];
            let grid_char = grid[start_row + pr][start_col + pc];

            if let Some(pat_char) = pat_char {
                if pat_char == grid_char {
                    continue;
                }
                return false;
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use advent_of_code::utils::matrix::{matrix_diff, skew_matrix_by};
    use advent_of_code::utils::string::{deformat_string, CharsMatrixToString, StringToCharsMatrix};
    use super::*;

    #[test]
    fn matches_pattern_test() {
        let input = deformat_string("
            ABC
            123
            ABC
        ").to_chars_matrix();
        let pattern = vec![
            vec![None, Some('B'), None],
            vec![None, None, Some('3')],
            vec![None, Some('B'), None],
        ];
        assert!(matches_pattern(&input, &pattern, 0, 0, 3, 3, 3, 3));
        assert!(!matches_pattern(&input, &pattern, 1, 1, 3, 3, 3, 3));
    }

    #[test]
    fn test() -> Result<(), Box<dyn std::error::Error>> {
        let input = deformat_string("
            MMMSXXMASM
            MSAMXMSMSA
            AMXSXMAAMM
            MSAMASMSMX
            XMASAMXAMM
            XXAMMXXAMA
            SMSMSASXSS
            SAXAMASAAA
            MAMMMXMMMM
            MXMXAXMASX
        ");
        let expected = deformat_string("
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
        ");
        let grid = input.to_chars_matrix();
        let expected = expected.to_chars_matrix();

        let base_patterns = vec![
            PATTERN.clone(), // XMAS
            /*
                X
                 M
                  A
                   S
             */
            skew_matrix_by(&PATTERN, 1),
        ];
        let mut patterns = vec![];
        patterns.extend(base_patterns.clone());
        patterns.extend(base_patterns.iter().map(rotate_matrix_90_degrees).collect::<Vec<_>>());
        patterns.extend(base_patterns.iter().map(rotate_matrix_180_degrees).collect::<Vec<_>>());
        patterns.extend(base_patterns.iter().map(rotate_matrix_270_degrees).collect::<Vec<_>>());

        println!("Patterns:\n");
        patterns.iter().for_each(|pattern| {
            println!("{}", pattern.to_string());
        });
        println!();

        let rows = grid.len();
        let cols = grid[0].len();

        let mut individual_match_visualization_data = grid.clone()
            .into_iter()
            .map(|line| line
                .into_iter()
                .map(|_| ' ')
                .collect::<Vec<_>>()
            )
            .collect::<Vec<_>>();

        let mut match_visualization_data = grid.clone()
            .into_iter()
            .map(|line| line
                .into_iter()
                .map(|_| '.')
                .collect::<Vec<_>>()
            )
            .collect::<Vec<_>>();

        let mut matches = HashSet::new();

        let mut count = 0;
        for pattern in patterns {
            let pat_rows = pattern.len();
            let pat_cols = pattern[0].len();
            for row in 0..rows {
                for col in 0..cols {
                    if matches_pattern(&grid, &pattern, row, col, rows, cols, pat_rows, pat_cols) {
                        count += 1;
                        let mut debug_individual_match_visualization = individual_match_visualization_data.clone();
                        for (y, line) in pattern.iter().enumerate() {
                            for (x, char) in line.iter().enumerate() {
                                if char.is_none() {
                                    continue;
                                }
                                let y = row + y;
                                let x = col + x;
                                match_visualization_data[y][x] = grid[y][x];
                                debug_individual_match_visualization[y][x] = grid[y][x];
                            }
                        }
                        if !matches.insert(debug_individual_match_visualization.to_string()) {
                            panic!("duplicate match");
                        }
                        println!("Pattern\n{}", pattern.to_string());
                        println!("Match\n{}", debug_individual_match_visualization.to_string());
                    }
                }
            }
        }
        println!("{}", match_visualization_data.to_string());
        println!("diff:\n{}", matrix_diff(&expected, &match_visualization_data, |_, b| *b).to_string());
        assert_eq!(match_visualization_data, expected);
        assert_eq!(count, 18);
        Ok(())
    }
}
