/// incorrect

use std::iter::Rev;
use std::ops::Range;
use std::slice::Iter;
use regex::{Match, Regex};
use smart_default::SmartDefault;

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

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Coord {
    x: usize,
    y: usize,
}

impl Coord {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct CoordIteratorResult {
    coord: Coord,
    wrapped: bool,
}

impl CoordIteratorResult {
    fn new(coord: Coord, is_new: bool) -> Self {
        Self { coord, wrapped: is_new }
    }
}

impl From<(usize, usize, bool)> for CoordIteratorResult {
    fn from((x, y, wrapped): (usize, usize, bool)) -> Self {
        Self { coord: Coord::new(x, y), wrapped }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct ConsumableFlag(Option<()>);
impl ConsumableFlag {
    const fn new_unset() -> Self {
        Self(None)
    }
    const fn new_set() -> Self {
        Self(Some(()))
    }
    fn consume(&mut self) -> bool {
        self.0.take().is_some()
    }
    fn set(&mut self) {
        self.0 = Some(());
    }
    fn clear(&mut self) {
        self.0 = None;
    }
    fn is_set(&self) -> bool {
        self.0.is_some()
    }
}
#[derive(SmartDefault, Debug, Clone, Copy)]
struct DiagonalIterator {
    width: usize,
    height: usize,
    /// Current diagonal index (front)
    diag_index: usize,
    /// Current coordinate index in the diagonal (front)
    coord_index: usize,
    /// Current diagonal index (back)
    back_diag_index: usize,
    /// Current coordinate index in the diagonal (back)
    back_coord_index: usize,
    /// Wrap flag to indicate that we've wrapped around
    #[default(ConsumableFlag::new_set())]
    wrap_flag: ConsumableFlag,
    anti: bool,
}

impl DiagonalIterator {
    fn new(width: usize, height: usize, anti: bool) -> Self {
        Self {
            width,
            height,
            anti,
            back_diag_index: width + height - 1, // Start at the last diagonal
            ..Default::default()
        }
    }
}

impl Iterator for DiagonalIterator {
    type Item = CoordIteratorResult;

    fn next(&mut self) -> Option<Self::Item> {
        while self.diag_index < self.width + self.height - 1 {
            let x = self.coord_index;
            let y = self.diag_index.saturating_sub(x);

            let x = if self.anti {
                self.width - 1 - x // Flip `x` for anti-diagonal.
            } else {
                x
            };

            if x < self.width && y < self.height {
                if y == 0 {
                    self.wrap_flag.set();
                }
                let result = CoordIteratorResult::new(Coord::new(x, y), self.wrap_flag.consume());
                self.coord_index += 1;
                return Some(result);
            } else {
                // Move to the next diagonal
                self.diag_index += 1;
                self.coord_index = 0;
            }
        }
        None
    }
}

impl DoubleEndedIterator for DiagonalIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        while self.back_diag_index > 0 {
            let x = self.back_coord_index;
            let y = self.back_diag_index.saturating_sub(x);

            let x = if self.anti {
                self.width - 1 - x // Flip `x` for anti-diagonal.
            } else {
                x
            };

            if x < self.width && y < self.height {
                if y == self.width + self.height - 1 {
                    self.wrap_flag.set();
                }
                let result = CoordIteratorResult::new(Coord::new(x, y), self.wrap_flag.consume());
                self.back_coord_index -= 1;
                return Some(result);
            } else {
                // Move to the next diagonal
                self.back_diag_index -= 1;
                self.back_coord_index = self.width + self.height - 1;
            }
        }
        None
    }
}

#[derive(Debug, Copy, Clone, Default)]
enum Major {
    #[default]
    Row,
    Col,
}

impl Major {
    fn iter(self, width: usize, height: usize) -> MajorIterator {
        MajorIterator::new(self, width, height)
    }
}

#[derive(SmartDefault, Debug, Clone, Copy)]
struct MajorIterator {
    width: usize,
    height: usize,
    current_row: usize,
    current_col: usize,
    major: Major,
    #[default(ConsumableFlag::new_set())]
    wrap_flag: ConsumableFlag,
    back_row: usize,
    back_col: usize,
}

impl MajorIterator {
    fn new(major: Major, width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            major,
            back_row: height - 1,
            back_col: width - 1,
            ..Default::default()
        }
    }
    /// passed y
    fn passed_primary(&self) -> bool {
        match self.major {
            Major::Row => self.current_row > self.back_row,
            Major::Col => self.current_col > self.back_col,
        }
    }
    /// passed x
    fn passed_secondary(&self) -> bool {
        match self.major {
            Major::Row => self.current_col > self.back_col,
            Major::Col => self.current_row > self.back_row,
        }
    }
    /// same y
    fn same_set(&self) -> bool {
        match self.major {
            Major::Row => self.current_row == self.back_row,
            Major::Col => self.current_col == self.back_col,
        }
    }
    fn is_done(&self) -> bool {
        // determine if we have passed the other iter side
        let has_passed_other_side = match self.major {
            Major::Row => self.passed_primary() || self.same_set() && self.passed_secondary(),
            Major::Col => self.passed_primary() || self.same_set() && self.passed_secondary(),
        };
        if has_passed_other_side {
            true
        } else {
            // is rev done?
            self.wrap_flag.is_set() && self.back_col == 0 && self.back_row == 0
        }
    }
}

impl Iterator for MajorIterator {
    type Item = CoordIteratorResult;

    fn next(&mut self) -> Option<Self::Item> {
        // Stop if we have traversed all indices
        if self.is_done() {
            return None;
        }

        // Save the current coordinate to yield
        let coord = Coord::new(self.current_row, self.current_col);

        // Determine which index to increment and its maximum bound
        let (ix, iy, ix_max) = match self.major {
            Major::Row => (&mut self.current_col, &mut self.current_row, self.width),
            Major::Col => (&mut self.current_row, &mut self.current_col, self.height),
        };

        let result = CoordIteratorResult::new(coord, self.wrap_flag.consume());

        // Increment the current index (`ix`) and wrap around if necessary
        *ix += 1;
        if *ix >= ix_max {
            *ix = 0; // Wrap around
            *iy += 1; // Move to the next row/column
            self.wrap_flag.set(); // Reset wrap flag
        }

        Some(result)
    }
}

impl DoubleEndedIterator for MajorIterator {
    fn next_back(&mut self) -> Option<Self::Item> {
        // Stop if we have traversed all indices
        if self.is_done() {
            return None;
        }

        // Save the current coordinate to yield
        let coord = Coord::new(self.back_row, self.back_col);

        // Determine which index to decrement and its maximum bound
        let (ix, iy, ix_max) = match self.major {
            Major::Row => (&mut self.back_col, &mut self.back_row, self.width - 1),
            Major::Col => (&mut self.back_row, &mut self.back_col, self.height - 1),
        };

        let result = CoordIteratorResult::new(coord, self.wrap_flag.consume());

        // Decrement the current index (`ix`) and wrap around if necessary
        if *ix == 0 && *iy > 0 {
            *ix = ix_max; // Wrap around
            *iy = iy.saturating_sub(1); // Move to the previous row/column
            self.wrap_flag.set(); // Reset wrap flag
        }
        else if *ix == 0 && *iy == 0 {
            // TODO: hacky since usize can't drop below 0
            // Set wrap flag so next iteration we know we've already visited 0,0
            self.wrap_flag.set();
        }
        else if *ix > 0 {
            *ix -= 1;
        }

        Some(result)
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

    for CoordIteratorResult { coord: Coord { x, y }, wrapped }  in Major::Row.iter(width, height) {

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
    use super::*;

    fn debug_print_coord_iterator_result(results: impl IntoIterator<Item=CoordIteratorResult> + Clone) {
        let width = results.clone().into_iter().map(|r| r.coord.x + 1).max().unwrap();
        let height = results.clone().into_iter().map(|r| r.coord.y + 1).max().unwrap();
        if width > 9 || height > 9 {
            panic!("expected width and height to be less than 10")
        }
        let mut data = vec![vec![" ".to_string(); width]; height];
        let mut ix = 0;
        for CoordIteratorResult { coord: Coord { x,y }, wrapped } in results {
            if wrapped {
                ix += 1;
            }
            println!("{ix}: {x},{y}");
            let current = data[y][x].clone();
            data[y][x] = if current == " " {
                ix.to_string()
            } else {
                format!("{current},{ix}").parse().unwrap()
            }
        }
        for row in data {
            for (ix, col) in row.iter().enumerate() {
                let ix = ix + 1;
                print!("{:<8}", format!("({col}).{ix}"));
            }
            println!();
        }
    }

    #[test]
    fn row_test() {
        let mut iter = Major::Row.iter(2, 2);
        assert_eq!(iter.next(), Some((0, 0, true).into()));
        assert_eq!(iter.next(), Some((0, 1, false).into()));
        assert_eq!(iter.next(), Some((1, 0, true).into()));
        assert_eq!(iter.next(), Some((1, 1, false).into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn row_rev_test() {
        let mut iter = Major::Row.iter(2, 2).rev();
        assert_eq!(iter.next(), Some((1, 1, true).into()));
        assert_eq!(iter.next(), Some((1, 0, false).into()));
        assert_eq!(iter.next(), Some((0, 1, true).into()));
        assert_eq!(iter.next(), Some((0, 0, false).into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn col_test() {
        let mut iter = Major::Col.iter(2, 2);
        assert_eq!(iter.next(), Some((0, 0, true).into()));
        assert_eq!(iter.next(), Some((1, 0, false).into()));
        assert_eq!(iter.next(), Some((0, 1, true).into()));
        assert_eq!(iter.next(), Some((1, 1, false).into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn col_rev_test() {
        let mut iter = Major::Col.iter(2, 2).rev();
        assert_eq!(iter.next(), Some((1, 1, true).into()));
        assert_eq!(iter.next(), Some((0, 1, false).into()));
        assert_eq!(iter.next(), Some((1, 0, true).into()));
        assert_eq!(iter.next(), Some((0, 0, false).into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn diag_test() {
        let mut iter = DiagonalIterator::new(2, 2, false);
        debug_print_coord_iterator_result(iter.clone());
        assert_eq!(iter.next(), Some((0, 0, true).into()));
        assert_eq!(iter.next(), Some((1, 0, true).into()));
        assert_eq!(iter.next(), Some((0, 1, false).into()));
        assert_eq!(iter.next(), Some((1, 1, true).into()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn diag_rev_test() {
        let mut iter = DiagonalIterator::new(2, 2, false).rev();
        for CoordIteratorResult { coord: Coord {x, y}, wrapped } in iter.clone() {
            println!("{x},{y},{wrapped}");
        }
        // assert_eq!(iter.next(), Some((1, 1, true).into()));
        // assert_eq!(iter.next(), Some((0, 1, false).into()));
        // assert_eq!(iter.next(), Some((1, 0, true).into()));
        // assert_eq!(iter.next(), Some((0, 0, true).into()));
        // assert_eq!(iter.next(), None);
    }
}
