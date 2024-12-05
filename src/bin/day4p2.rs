use lazy_static::lazy_static;
use advent_of_code::utils::matrix::{rotate_matrix_180_degrees, rotate_matrix_270_degrees, rotate_matrix_90_degrees};

lazy_static! {
    static ref PATTERN: Vec<Vec<Option<char>>> = {
        vec![
            vec![Some('M'), None, Some('S')],
            vec![None, Some('A'), None],
            vec![Some('M'), None, Some('S')],
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

    let base_patterns = vec![PATTERN.clone()];
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
    use advent_of_code::utils::string::{deformat_string, StringToCharsMatrix};
    use super::*;

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
            .M.S......
            ..A..MSMS.
            .M.S.MAA..
            ..A.ASMSM.
            .M.S.M....
            ..........
            S.S.S.S.S.
            .A.A.A.A..
            M.M.M.M.M.
            ..........
        ");
        let grid = input.to_chars_matrix();
        let expected = expected.to_chars_matrix();

        let base_patterns = vec![PATTERN.clone()];
        let mut patterns = vec![];
        patterns.extend(base_patterns.clone());
        patterns.extend(base_patterns.iter().map(rotate_matrix_90_degrees).collect::<Vec<_>>());
        patterns.extend(base_patterns.iter().map(rotate_matrix_180_degrees).collect::<Vec<_>>());
        patterns.extend(base_patterns.iter().map(rotate_matrix_270_degrees).collect::<Vec<_>>());

        let rows = grid.len();
        let cols = grid[0].len();

        let mut match_visualization_data = grid.clone()
            .into_iter()
            .map(|line| line
                .into_iter()
                .map(|_| '.')
                .collect::<Vec<_>>()
            )
            .collect::<Vec<_>>();

        let mut count = 0;
        for pattern in patterns {
            let pat_rows = pattern.len();
            let pat_cols = pattern[0].len();
            for row in 0..rows {
                for col in 0..cols {
                    if matches_pattern(&grid, &pattern, row, col, rows, cols, pat_rows, pat_cols) {
                        count += 1;
                        for (y, line) in pattern.iter().enumerate() {
                            for (x, char) in line.iter().enumerate() {
                                if char.is_none() {
                                    continue;
                                }
                                let y = row + y;
                                let x = col + x;
                                match_visualization_data[y][x] = grid[y][x];
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(match_visualization_data, expected);
        assert_eq!(count, 9);
        Ok(())
    }
}
