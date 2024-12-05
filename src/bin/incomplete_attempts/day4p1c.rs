/// correct
/// TODO: fix/cleanup

use std::collections::VecDeque;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Major {
    Row,
    Col,
}

impl Major {
    fn iter(self, cols: usize, rows: usize) -> MatrixMajorIterator {
        MatrixMajorIterator::new(self, cols, rows)
    }
}

#[derive(Debug, Clone, Copy)]
struct MatrixMajorIterator {
    rows: usize,
    cols: usize,
    forward_row: usize,
    forward_col: usize,
    major: Major,
    back_row: usize,
    back_col: usize,
}

impl MatrixMajorIterator {
    fn new(major: Major, rows: usize, cols: usize) -> Self {
        let (back_row, back_col) = match major {
            Major::Row => (rows - 1, cols),
            Major::Col => (rows, cols - 1),
        };
        Self {
            rows,
            cols,
            forward_row: 0,
            forward_col: 0,
            major,
            back_row,
            back_col,
        }
    }
    /// passed y
    fn passed_primary(&self) -> bool {
        match self.major {
            Major::Row => self.forward_row > self.back_row,
            Major::Col => self.forward_col > self.back_col,
        }
    }
    /// passed x
    fn passed_secondary(&self) -> bool {
        match self.major {
            Major::Row => self.forward_col > self.back_col,
            Major::Col => self.forward_row > self.back_row,
        }
    }
    /// same y
    fn same_set(&self) -> bool {
        match self.major {
            Major::Row => self.forward_row == self.back_row,
            Major::Col => self.forward_col == self.back_col,
        }
    }
    fn is_done(&self) -> bool {
        // determine if we have passed the other iter side
        if self.passed_primary() || self.same_set() && self.passed_secondary() {
            true
        } else {
            // is rev done?
            self.back_col == 0 && self.back_row == 0
        }
    }
    fn is_start(&self, rev: bool) -> bool {
        if rev {
            match self.major {
                Major::Row => self.back_col == self.cols - 1,
                Major::Col => self.back_row == self.rows - 1,
            }
        } else {
            match self.major {
                Major::Row => self.forward_col == 0,
                Major::Col => self.forward_row == 0,
            }
        }
    }
    fn coordinate(&self, col: usize, row: usize) -> Option<(usize, usize)> {
        let (x, y, cx, cy) = match self.major {
            Major::Row => (col, row, self.cols, self.rows),
            Major::Col => (col, row, self.rows, self.cols),
        };
        if x < cx && y < cy {
            Some((x, y))
        } else {
            None
        }

    }
}

impl Iterator for MatrixMajorIterator {
    type Item = ((usize, usize), bool);

    fn next(&mut self) -> Option<Self::Item> {
        // Stop if we have traversed all indices
        if self.is_done() {
            return None;
        }

        // Save the current coordinate to yield
        let coord = self.coordinate(self.forward_col, self.forward_row);
        let is_start = self.is_start(false);

        // Determine which index to increment and its maximum bound
        let (ix, iy, ix_max) = match self.major {
            Major::Row => (&mut self.forward_col, &mut self.forward_row, self.cols),
            Major::Col => (&mut self.forward_row, &mut self.forward_col, self.rows),
        };

        // Increment the current index (`ix`) and wrap around if necessary
        *ix += 1;
        if *ix >= ix_max {
            *ix = 0; // Wrap around
            *iy += 1; // Move to the next row/column
        }

        if let Some(coord)  = coord {
            Some((coord, is_start))
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for MatrixMajorIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        // Stop if we have traversed all indices
        if self.is_done() {
            return None;
        }

        // TODO: we need to decrement right away because we start outside the bounds
        // we do that because we need to check if either side of the iterator have passed each other
        // potentially just changing the condition there would remove the requirement of decrementing
        let target = match self.major {
            Major::Row => { &mut self.back_col },
            Major::Col => { &mut self.back_row },
        };
        let original_value = *target;
        *target = original_value.saturating_sub(1);

        // Save the current coordinate to yield
        let coord = self.coordinate(self.back_col, self.back_row);
        let is_start = self.is_start(true);

        // Determine which index to decrement and its maximum bound
        let (ix, iy, ix_max) = match self.major {
            Major::Row => (&mut self.back_col, &mut self.back_row, self.cols),
            Major::Col => (&mut self.back_row, &mut self.back_col, self.rows),
        };

        // Decrement the current index (`ix`) and wrap around if necessary
        // *ix = ix.saturating_sub(1);
        // TODO: see todo above
        if *ix == 0 && *iy > 0 {
            *ix = ix_max; // Wrap around
            *iy = iy.saturating_sub(1); // Move to the previous row/column
        }

        if let Some(coord) = coord {
            Some((coord, is_start))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MatrixDiagonalCoordIterator {
    rows: usize,
    cols: usize,
    forward_diagonal_index: usize,
    forward_in_diagonal_index: usize,
    back_diagonal_index: usize,
    back_in_diagonal_index: usize,
    is_anti: bool,
}

impl MatrixDiagonalCoordIterator {
    pub fn new(rows: usize, cols: usize, is_anti: bool) -> Self {
        Self {
            rows,
            cols,
            forward_diagonal_index: 0,
            forward_in_diagonal_index: 0,
            back_diagonal_index: rows + cols - 1,
            back_in_diagonal_index: 0,
            is_anti,
        }
    }

    fn diagonal_length(&self, diagonal_index: usize) -> usize {
        let compare_limit = if self.is_anti {
            self.rows
        } else {
            self.cols
        };

        if diagonal_index < compare_limit {
            diagonal_index + 1
        } else {
            self.rows + self.cols - 1 - diagonal_index
        }
    }

    fn coordinate(&self, diagonal_index: usize, in_diagonal_index: usize) -> Option<(usize, usize)> {
        let compare_limit = if self.is_anti {
            self.cols
        } else {
            self.rows
        };

        let primary = if diagonal_index < compare_limit {
            in_diagonal_index
        } else {
            diagonal_index - compare_limit + 1 + in_diagonal_index
        };
        let secondary = diagonal_index - primary;

        let (row, col) = if self.is_anti {
            (primary, secondary)
        } else {
            (secondary, primary)
        };

        // Validate bounds
        if row < self.rows && col < self.cols {
            Some((row, col))
        } else {
            None
        }
    }

    fn is_done(&self) -> bool {
        self.forward_diagonal_index > self.back_diagonal_index
            || (self.forward_diagonal_index == self.back_diagonal_index
            && self.forward_in_diagonal_index >= self.back_in_diagonal_index)
    }
}

impl Iterator for MatrixDiagonalCoordIterator {
    type Item = ((usize, usize), bool);

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_done() || self.forward_diagonal_index >= self.rows + self.cols - 1 {
            return None;
        }

        let is_new_diagonal = self.forward_in_diagonal_index == 0;

        if let Some(coord) = self.coordinate(self.forward_diagonal_index, self.forward_in_diagonal_index) {
            self.forward_in_diagonal_index += 1;
            if self.forward_in_diagonal_index >= self.diagonal_length(self.forward_diagonal_index) {
                self.forward_diagonal_index += 1;
                self.forward_in_diagonal_index = 0;
            }
            Some((coord, is_new_diagonal))
        } else {
            None
        }
    }
}

impl DoubleEndedIterator for MatrixDiagonalCoordIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.is_done() || self.back_diagonal_index == 0 && self.back_in_diagonal_index == 0 {
            return None;
        }

        if self.back_in_diagonal_index > 0 {
            self.back_in_diagonal_index -= 1;
        } else {
            self.back_diagonal_index -= 1;
            self.back_in_diagonal_index = self.diagonal_length(self.back_diagonal_index) - 1;
        }

        let is_new_diagonal = self.back_in_diagonal_index == self.diagonal_length(self.back_diagonal_index) - 1;
        self.coordinate(self.back_diagonal_index, self.back_in_diagonal_index).map(|coord| (coord, is_new_diagonal))
    }
}

#[derive(Debug, Clone, Default)]
struct RollingWindowWithCoords<T> {
    window_size: usize,
    data: VecDeque<T>,
    coords: VecDeque<(usize, usize)>,
}

impl<T: Default + PartialEq> RollingWindowWithCoords<T> {
    fn new(window_size: usize) -> Self {
        Self {
            window_size,
            ..Default::default()
        }
    }
    fn push_back(&mut self, item: T, coord: (usize, usize)) {
        self.data.push_back(item);
        self.coords.push_back(coord);
        while self.data.len() > self.window_size {
            self.data.pop_front();
            self.coords.pop_front();
        }
    }
    fn clear(&mut self) {
        self.data.clear();
        self.coords.clear();
    }
    fn len(&self) -> usize {
        self.data.len()
    }
    fn ends_with(&self, needle: &[T]) -> bool {
        if self.data.len() < needle.len() {
            return false;
        }
        let mut iter = self.data.iter().rev();
        for c in needle.iter().rev() {
            if Some(c) != iter.next() {
                return false
            }
        }
        true
    }
    fn data(&self) -> Vec<&T> {
        self.data.iter().collect()
    }
    fn consume(&self) -> Vec<(&T, &(usize, usize))> {
        self.data.iter().zip(self.coords.iter()).collect()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lines = advent_of_code::read_input(4);
    let mut data = vec![];
    for input_line in lines {
        let input_line = input_line?;
        let input_line = input_line.trim();
        data.push(input_line.chars().collect::<Vec<_>>());
    }
    let height = data.len();
    let width = data.first().unwrap().len();

    fn update<T>(output: &mut Vec<Vec<T>>, value: T, coord: (usize, usize)) {
        let (x,y) = coord;
        let row = output.get_mut(y).unwrap();
        *row.get_mut(x).unwrap() = value;
    };

    let needle = "XMAS".chars().collect::<Vec<_>>();
    let needle_reverse = needle.iter().cloned().rev().collect::<Vec<_>>();

    let mut window = RollingWindowWithCoords::new(needle.len());
    let mut count = 0;

    let mut process = |data: &mut Vec<Vec<char>>, ((x,y), wrapped)| {
        if wrapped {
            window.clear()
        }
        let line: &Vec<char> = data.get(y).unwrap();
        let char = *line.get(x).unwrap();
        window.push_back(char, (x, y));
        if window.len() == needle.len() {
            let consume =
                if needle.ends_with(&[char]) {
                    window.ends_with(needle.as_slice())
                } else if needle_reverse.ends_with(&[char]) {
                    window.ends_with(needle_reverse.as_slice())
                } else {
                    false
                };

            if consume {
                count += 1;
                window.consume();
            }
        }
    };

    // println!("Rows");
    for result in MatrixMajorIterator::new(Major::Row, height, width) {
        process(&mut data, result);
    }

    // println!("Cols");
    for result in MatrixMajorIterator::new(Major::Col, height, width) {
        process(&mut data, result);
    }

    // println!("Diag");
    for result in MatrixDiagonalCoordIterator::new(height, width, false) {
        process(&mut data, result);
    }

    fn rotate_matrix_90_degrees<T: Copy + Default>(matrix: Vec<Vec<T>>) -> Vec<Vec<T>> {
        let rows = matrix.len();
        let cols = matrix[0].len();

        // Create a new matrix with reversed dimensions
        let mut rotated = vec![vec![T::default(); rows]; cols];

        // Populate the rotated matrix
        for i in 0..rows {
            for j in 0..cols {
                rotated[j][rows - i - 1] = matrix[i][j];
            }
        }

        rotated
    }

    data = rotate_matrix_90_degrees(data);
    for result in MatrixDiagonalCoordIterator::new(height, width, false) {
        process(&mut data, result);
    }

    // TODO: broken
    // println!("Diag Anti");
    // for result in MatrixDiagonalCoordIterator::new(height, width, true) {
    //     process(&mut data, result);
    // }

    let answer = count;
    println!("Answer: {answer}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use super::*;

    fn debug_print_coord_iterator_result(results: impl IntoIterator<Item=((usize, usize), bool)> + Clone) {
        let width = results.clone().into_iter().map(|((x, ..), ..)| x + 1).max().unwrap();
        let height = results.clone().into_iter().map(|((.., y), ..)| y + 1).max().unwrap();
        if width > 9 || height > 9 {
            panic!("expected width and height to be less than 10")
        }
        let mut data = vec![vec![" ".to_string(); width]; height];
        let mut ix = 0;
        let mut iy = 0;
        for ((x, y), wrapped) in results {
            if wrapped {
                ix += 1;
                iy = 0;
            }
            iy += 1;
            let access_order_string = format!("{ix}.{iy}");
            println!("- {access_order_string}: {x},{y}");
            let current = data[y][x].clone();
            data[y][x] = if current == " " {
                format!("{access_order_string}")
            } else {
                format!("{current},{ix}").parse().unwrap()
            }
        }
        println!();
        for row in data {
            for col in row.iter() {
                print!("{col:<8}");
            }
            println!();
        }
    }

    #[test]
    fn row_test() {
        let mut iter = Major::Row.iter(2, 2);
        assert_eq!(iter.next(), Some(((0, 0), true)));
        assert_eq!(iter.next(), Some(((1, 0), false)));
        assert_eq!(iter.next(), Some(((0, 1), true)));
        assert_eq!(iter.next(), Some(((1, 1), false)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn row_rev_test() {
        let mut iter = Major::Row.iter(2, 2).rev();
        assert_eq!(iter.next(), Some(((1, 1), true)));
        assert_eq!(iter.next(), Some(((0, 1), false)));
        assert_eq!(iter.next(), Some(((1, 0), true)));
        assert_eq!(iter.next(), Some(((0, 0), false)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn row_double_ended_test() {
        let mut iter = Major::Row.iter(2, 2);
        assert_eq!(iter.next(), Some(((0, 0), true)));
        assert_eq!(iter.next_back(), Some(((1, 1), true)));
        assert_eq!(iter.next(), Some(((1, 0), false)));
        assert_eq!(iter.next_back(), Some(((0, 1), false)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn col_test() {
        let mut iter = Major::Col.iter(2, 2);
        debug_print_coord_iterator_result(iter.clone());
        assert_eq!(iter.next(), Some(((0, 0), true)));
        assert_eq!(iter.next(), Some(((0, 1), false)));
        assert_eq!(iter.next(), Some(((1, 0), true)));
        assert_eq!(iter.next(), Some(((1, 1), false)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn col_rev_test() {
        let mut iter = Major::Col.iter(2, 2).rev();
        assert_eq!(iter.next(), Some(((1, 1), true)));
        assert_eq!(iter.next(), Some(((1, 0), false)));
        assert_eq!(iter.next(), Some(((0, 1), true)));
        assert_eq!(iter.next(), Some(((0, 0), false)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn diag_test() {
        let mut iter = MatrixDiagonalCoordIterator::new(2, 2, false);
        debug_print_coord_iterator_result(iter.clone());
        assert_eq!(iter.next(), Some(((0, 0), true)));
        assert_eq!(iter.next(), Some(((0, 1), true)));
        assert_eq!(iter.next(), Some(((1, 0), false)));
        assert_eq!(iter.next(), Some(((1, 1), true)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn diag_anti_test() {
        let mut iter = MatrixDiagonalCoordIterator::new(2, 2, true);
        debug_print_coord_iterator_result(iter.clone());
        assert_eq!(iter.next(), Some(((1, 0), true)));
        assert_eq!(iter.next(), Some(((1, 1), true)));
        assert_eq!(iter.next(), Some(((0, 0), false)));
        assert_eq!(iter.next(), Some(((0, 1), true)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn diag_rev_test() {
        let mut iter = MatrixDiagonalCoordIterator::new(2, 2, false).rev();
        assert_eq!(iter.next(), Some(((1, 1), true)));
        assert_eq!(iter.next(), Some(((1, 0), true)));
        assert_eq!(iter.next(), Some(((0, 1), false)));
        assert_eq!(iter.next(), Some(((0, 0), true)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn diag_anti_rev_test() {
        let mut iter = MatrixDiagonalCoordIterator::new(2, 2, true).rev();
        assert_eq!(iter.next(), Some(((0, 1), true)));
        assert_eq!(iter.next(), Some(((0, 0), true)));
        assert_eq!(iter.next(), Some(((1, 1), false)));
        assert_eq!(iter.next(), Some(((1, 0), true)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn diag_double_ended_test() {
        let mut iter = MatrixDiagonalCoordIterator::new(2, 2, false);
        assert_eq!(iter.next(), Some(((0, 0), true)));
        assert_eq!(iter.next_back(), Some(((1, 1), true)));
        assert_eq!(iter.next(), Some(((1, 0), true)));
        assert_eq!(iter.next_back(), Some(((0, 1), true)));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    fn example() {
        fn cleanup(input: &str) -> String {
            input
                .lines()
                .map(|line| line.trim().to_string() + "\n")
                .filter(|line| !line.is_empty())
                .collect()
        }
        let input = cleanup("
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
        let expected = cleanup("
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
        let mut data = [[' '; 10]; 10];
        let mut output = data.map(|line| line.to_vec()).into_iter().collect::<Vec<_>>();
        for (iy, line) in input.trim().lines().enumerate() {
            for (ix, char) in line.chars().enumerate() {
                data[iy][ix] = char;
            }
        }
        let mut data: Vec<Vec<char>> = data.map(|line| line.to_vec()).into_iter().collect::<Vec<_>>();


        fn update<T>(output: &mut Vec<Vec<T>>, value: T, coord: (usize, usize)) {
            let (x,y) = coord;
            let row = output.get_mut(y).unwrap();
            *row.get_mut(x).unwrap() = value;
        };

        let needle = "XMAS".chars().collect::<Vec<_>>();
        let needle_reverse = needle.iter().cloned().rev().collect::<Vec<_>>();

        #[derive(Debug, Clone, Default)]
        struct RollingWindowWithCoords<T> {
            window_size: usize,
            data: VecDeque<T>,
            coords: VecDeque<(usize, usize)>,
        }

        impl<T: Default + PartialEq> RollingWindowWithCoords<T> {
            fn new(window_size: usize) -> Self {
                Self {
                    window_size,
                    ..Default::default()
                }
            }
            fn push_back(&mut self, item: T, coord: (usize, usize)) {
                self.data.push_back(item);
                self.coords.push_back(coord);
                while self.data.len() > self.window_size {
                    self.data.pop_front();
                    self.coords.pop_front();
                }
            }
            fn clear(&mut self) {
                self.data.clear();
                self.coords.clear();
            }
            fn len(&self) -> usize {
                self.data.len()
            }
            fn ends_with(&self, needle: &[T]) -> bool {
                if self.data.len() < needle.len() {
                    return false;
                }
                let mut iter = self.data.iter().rev();
                for c in needle.iter().rev() {
                    if Some(c) != iter.next() {
                        return false
                    }
                }
                true
            }
            fn data(&self) -> Vec<&T> {
                self.data.iter().collect()
            }
            fn consume(&self) -> Vec<(&T, &(usize, usize))> {
                self.data.iter().zip(self.coords.iter()).collect()
            }
        }

        let mut window = RollingWindowWithCoords::new(needle.len());
        let mut count = 0;

        let mut process = |data: &mut Vec<Vec<char>>, output: &mut Vec<Vec<char>>, ((x,y), wrapped)| {
            if wrapped {
                window.clear()
            }
            let line: &Vec<char> = data.get(y).unwrap();
            let char = *line.get(x).unwrap();
            window.push_back(char, (x, y));
            // print!("{x},{y:} {wrapped:5}: {}", window.data().into_iter().map(|c| *c).collect::<String>());
            if window.len() == needle.len() {
                let consume =
                    if needle.ends_with(&[char]) {
                        window.ends_with(needle.as_slice())
                    } else if needle_reverse.ends_with(&[char]) {
                        window.ends_with(needle_reverse.as_slice())
                    } else {
                        false
                    };

                if consume {
                    // print!(" - found");
                    count += 1;
                    for (&char, &coord) in window.consume() {
                        update(output, char, coord);
                    }
                }
            }
            // println!();
        };

        let height = data.len();
        let width = data[0].len();

        // println!("Rows");
        for result in MatrixMajorIterator::new(Major::Row, height, width) {
            process(&mut data, &mut output, result);
        }

        // println!("Cols");
        for result in MatrixMajorIterator::new(Major::Col, height, width) {
            process(&mut data, &mut output, result);
        }

        // println!("Diag");
        for result in MatrixDiagonalCoordIterator::new(height, width, false) {
            process(&mut data, &mut output, result);
        }

        fn rotate_matrix_90_degrees<T: Copy + Default>(matrix: Vec<Vec<T>>) -> Vec<Vec<T>> {
            let rows = matrix.len();
            let cols = matrix[0].len();

            // Create a new matrix with reversed dimensions
            let mut rotated = vec![vec![T::default(); rows]; cols];

            // Populate the rotated matrix
            for i in 0..rows {
                for j in 0..cols {
                    rotated[j][rows - i - 1] = matrix[i][j];
                }
            }

            rotated
        }

        data = rotate_matrix_90_degrees(data);
        output = rotate_matrix_90_degrees(output);
        for result in MatrixDiagonalCoordIterator::new(height, width, false) {
            process(&mut data, &mut output, result);
        }
        for _ in 0..4 {
            data = rotate_matrix_90_degrees(data);
            output = rotate_matrix_90_degrees(output);

            data = rotate_matrix_90_degrees(data);
            output = rotate_matrix_90_degrees(output);

            data = rotate_matrix_90_degrees(data);
            output = rotate_matrix_90_degrees(output);
        }

        // TODO: broken
        // println!("Diag Anti");
        // for result in MatrixDiagonalCoordIterator::new(height, width, true) {
        //     process(&mut data, &mut output, result);
        // }

        for line in output {
            let line = line.iter().collect::<String>();
            println!("{line}");
        }

        println!("Count: {}", count);
    }
}
