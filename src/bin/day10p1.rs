use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;
use glam::{IVec2, UVec2};
use itertools::Itertools;
use smart_default::SmartDefault;
use advent_of_code::utils::matrix::{MatrixDetails, MatrixIterator};
use advent_of_code::utils::string::{deformat_string, StringToCharsMatrix};
use advent_of_code::utils::vec2::{IntoIVec2, IntoUsizeTuple, TryIntoUVec2};

fn main() -> anyhow::Result<()> {
    let mut lines = advent_of_code::read_input(10);
    let grid: Vec<Vec<_>> = lines.into_iter().map(|line| line.unwrap() + "\n")
        .collect::<String>()
        .to_chars_matrix()
        .into_iter()
        .enumerate()
        .map(|(y, row)| {
            row
                .into_iter()
                .enumerate()
                .map(|(x, c)| {
                    let digit = c
                        .to_digit(10)
                        .unwrap_or_else(|| panic!("failed to parse digit from {x},{y}: {c:?}"));
                    TrailPart {
                        pos: UVec2::new(x as u32, y as u32),
                        height: digit,
                    }
                })
                .collect()
        })
        .collect();

    let mut iterator = MultiDirectionalPathMatrixIterator::new(&grid);
    let paths = iterator.process().into_iter().map(|iterator| iterator.path);

    let sum = paths.map(|path| {
        (
            path.first().unwrap().pos,
            path.last().unwrap().pos,
        )
    }).collect::<HashSet<(UVec2, UVec2)>>().len();

    println!("sum: {}", sum);

    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
struct TrailPart {
    pos: UVec2,
    height: u32,
}

impl TrailPart {
    fn is_starting_pos(&self) -> bool {
        self.height == 0
    }
    fn is_ending_pos(&self) -> bool {
        self.height == 9
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn next(self) -> Direction {
        match self {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        }
    }
}

impl From<Direction> for IVec2 {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Up => IVec2::NEG_Y,
            Direction::Down => IVec2::Y,
            Direction::Left => IVec2::NEG_X,
            Direction::Right => IVec2::X,
        }
    }
}

#[derive(Debug, Clone, SmartDefault)]
struct DirectionIter {
    #[default(Direction::Up)]
    direction: Direction,
    seen: HashSet<Direction>,
}

impl DirectionIter {
    fn new(direction: Direction) -> Self {
        Self {
            direction,
            ..Default::default()
        }
    }
}

impl Iterator for DirectionIter {
    type Item = Direction;
    fn next(&mut self) -> Option<Self::Item> {
        let direction = self.direction;
        let next_direction = match direction {
            Direction::Up => Direction::Right,
            Direction::Right => Direction::Down,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
        };
        if self.seen.contains(&next_direction) {
            return None;
        }
        self.seen.insert(next_direction);
        self.direction = next_direction;
        Some(self.direction)
    }
}

pub trait PathItem {
    fn is_start(&self) -> bool;
    fn is_end(&self) -> bool;
    fn is_valid_next_item(&self, next: &Self) -> bool;
    fn get_pos(&self) -> UVec2;
    fn debug(&self) -> String;
}

impl PathItem for TrailPart {
    fn is_start(&self) -> bool {
        self.is_starting_pos()
    }
    fn is_end(&self) -> bool {
        self.is_ending_pos()
    }
    fn is_valid_next_item(&self, next: &Self) -> bool {
        let next_height = next.height;
        let current_height = self.height;
        next_height == current_height + 1
    }
    fn get_pos(&self) -> UVec2 {
        self.pos
    }
    fn debug(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectionalPathMatrixIterator<'a, T> where T: PathItem + Clone + PartialEq {
    // TODO: this should be &'a [&'a [T]] but it would be a pita to update the helper functions right now
    matrix: &'a Vec<Vec<T>>,
    matrix_details: MatrixDetails,
    path: Vec<&'a T>,

}

impl<'a, T> DirectionalPathMatrixIterator<'a, T> where T: PathItem + Clone + PartialEq {
    pub fn new(matrix: &'a Vec<Vec<T>>, starting_pos: UVec2) -> Self {
        let matrix_details = MatrixDetails::from_matrix(matrix);
        if !matrix_details.is_within_bounds(starting_pos) {
            panic!("starting pos not within bounds");
        }
        let item = &matrix[starting_pos.y as usize][starting_pos.x as usize];
        Self {
            matrix,
            matrix_details,
            path: vec![item],
        }
    }

    fn next(&mut self, direction: Direction) -> Option<&'a T> {
        let Some(&&ref last_item) = self.path.last() else {
            unreachable!();
        };
        if last_item.is_end() {
            return None;
        }
        let last_pos = last_item.get_pos();
        let last_pos = last_pos.into_ivec2();
        let next_pos = last_pos + IVec2::from(direction);
        let Ok(next_pos) = next_pos.try_into_uvec2() else {
            return None;
        };
        if !self.matrix_details.is_within_bounds(next_pos) {
            return None;
        }
        let (x, y) = next_pos.into_usize_tuple();
        let next_item = &self.matrix[y][x];
        if !last_item.is_valid_next_item(next_item) {
            return None;
        }
        if self.path.contains(&next_item) {
            return None;
        }
        self.path.push(next_item);
        Some(next_item)
    }

    fn starting_pos(&self) -> UVec2 {
        let Some(&&ref item) = self.path.first() else {
            unreachable!()
        };
        item.get_pos()
    }

    fn last_pos(&self) -> UVec2 {
        let Some(&&ref item) = self.path.last() else {
            unreachable!()
        };
        item.get_pos()
    }

    fn is_complete(&self) -> bool {
        let Some(&&ref last_item) = self.path.last() else {
            unreachable!();
        };
        last_item.is_end()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum IteratorState<T> {
    New(T),
    InProgress(T),
    Complete(T),
    Terminated(T),
}


#[derive(Debug, Clone)]
struct MultiDirectionalPathMatrixIterator<'a, T> where T: PathItem + Clone + PartialEq + Eq + Hash {
    matrix: &'a Vec<Vec<T>>,
    active_iterators: Vec<DirectionalPathMatrixIterator<'a, T>>,
    complete_iterators: Vec<DirectionalPathMatrixIterator<'a, T>>,
    terminated_iterators: Vec<DirectionalPathMatrixIterator<'a, T>>,
}

impl<'a, T> MultiDirectionalPathMatrixIterator<'a, T> where T: PathItem + Clone + PartialEq + Eq + Hash {
    fn new(matrix: &'a Vec<Vec<T>>) -> Self {
        let starting_positions = MatrixIterator::new(&matrix)
            .filter_map(|(pos, item)| {
                if item.is_start() {
                    Some(pos)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();


        let active_iterators = starting_positions
            .into_iter()
            .map(|pos| DirectionalPathMatrixIterator::new(&matrix, pos))
            .collect();

        Self {
            matrix,
            active_iterators,
            complete_iterators: vec![],
            terminated_iterators: vec![],
        }
    }
    fn process(mut self) -> Vec<DirectionalPathMatrixIterator<'a, T>> {
        loop {
            let mut iterator_updates = HashSet::new();
            while let Some(mut iterator) = self.active_iterators.pop() {
                // println!("path: {}", iterator.path.iter().map(|item| item.get_pos().to_string()).collect::<Vec<_>>().join(","));
                if iterator.is_complete() {
                    iterator_updates.insert(IteratorState::Complete(iterator));
                    continue;
                }
                let mut directions = DirectionIter::new(Direction::Up);
                // let last_item = iterator.path.last().unwrap();
                // let last_item_debug = last_item.debug();
                let mut cur_iter = iterator;
                for direction in directions {
                    let forked = cur_iter.clone();
                    if let Some(_) = cur_iter.next(direction) {
                        // let next_item_debug = item.debug();
                        // println!("{last_item_debug} -> {next_item_debug} ({direction:?})");
                        iterator_updates.insert(IteratorState::InProgress(cur_iter.clone()));
                    } else {
                        if cur_iter.is_complete() {
                            iterator_updates.insert(IteratorState::Complete(cur_iter.clone()));
                        } else {
                            iterator_updates.insert(IteratorState::Terminated(cur_iter.clone()));
                        }
                    };
                    cur_iter = forked;
                }
            }
            for iterator_state in iterator_updates {
                match iterator_state {
                    IteratorState::New(iter) => {
                        self.active_iterators.push(iter);
                    }
                    IteratorState::InProgress(iter) => {
                        self.active_iterators.push(iter);
                    }
                    IteratorState::Complete(iter) => {
                        self.complete_iterators.push(iter);
                    }
                    IteratorState::Terminated(iter) => {
                        self.terminated_iterators.push(iter);
                    }
                }
            }
            if self.active_iterators.is_empty() {
                return self.complete_iterators;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use advent_of_code::utils::matrix::MatrixIterator;
    use advent_of_code::utils::string::{deformat_string, StringToCharsMatrix};
    use super::*;

    #[derive(Debug, Copy, Clone)]
    enum DebugCell {
        Height(u32),
        End,
    }

    #[derive(Debug, Clone)]
    struct DebugGrid(Vec<Vec<Option<DebugCell>>>);

    impl DebugGrid {
        fn new(grid: &Vec<Vec<TrailPart>>) -> Self {
            Self(grid.iter().map(|row| row.iter().map(|col| {
                if col.is_ending_pos() {
                    Some(DebugCell::End)
                } else {
                    None
                }
            }).collect()).collect())
        }
        fn print(&self) -> std::fmt::Result {
            for row in self.0.iter() {
                for col in row.iter() {
                    print!("[ ");
                    if let Some(value) = col {
                        match value {
                            DebugCell::Height(value) => print!("{value}"),
                            DebugCell::End => print!("X"),
                        }
                    } else {
                        print!(" ");
                    };
                    print!(" ]");
                }
                println!();
            }
            println!();
            Ok(())
        }
    }

    #[test]
    fn examples() -> anyhow::Result<()> {
        let input = deformat_string("
            89010123
            78121874
            87430965
            96549874
            45678903
            32019012
            01329801
            10456732
        ");
        let grid: Vec<Vec<_>> = input
            .to_chars_matrix()
            .into_iter()
            .enumerate()
            .map(|(y, row)| {
                row
                    .into_iter()
                    .enumerate()
                    .map(|(x, c)| {
                        let digit = c
                            .to_digit(10)
                            .unwrap_or_else(|| panic!("failed to parse digit from {x},{y}: {c:?}"));
                        TrailPart {
                            pos: UVec2::new(x as u32, y as u32),
                            height: digit,
                        }
                    })
                    .collect()
            })
            .collect();

        let starting_positions = MatrixIterator::new(&grid).map(|(_, item)| item).cloned().filter(TrailPart::is_starting_pos).collect::<Vec<_>>();
        let ending_positions = MatrixIterator::new(&grid).map(|(_, item)| item).cloned().filter(TrailPart::is_ending_pos).collect::<Vec<_>>();

        let mut iterator = MultiDirectionalPathMatrixIterator::new(&grid);
        let paths = iterator.process().into_iter().map(|iterator| iterator.path);

        /*let mut map: HashMap<TrailPart, HashSet<Vec<TrailPart>>> = HashMap::new();
        for path in paths.into_iter().collect::<HashSet<_>>().into_iter() {
            // println!("{}", path.into_iter().map(|item| format!("{} ({})", item.pos, item.height)).collect::<Vec<_>>().join(" -> "));
            let Some(&&first) = path.first() else { unreachable!() };
            let entry = map.entry(first).or_default();
            entry.insert(path.into_iter().cloned().collect());
        }
        let mut sum = 0;
        for (start, paths) in map {
            // println!("{}", start.pos);
            sum += paths.iter().map(|path| path.iter().last().unwrap().pos).collect::<HashSet<UVec2>>().len();
            for path in paths {
                // println!("- {}", path.iter().map(|item| format!("{} ({})", item.pos, item.height)).collect::<Vec<_>>().join(" -> "));
                let mut grid = DebugGrid::new(&grid);
                for trail_part in path {
                    let (x, y) = trail_part.pos.into_usize_tuple();
                    grid.0[y][x] = Some(DebugCell::Height(trail_part.height));
                }
                // grid.print()?
            }
        }
        */

        let sum = paths.map(|path| {
            (
                path.first().unwrap().pos,
                path.last().unwrap().pos,
            )
        }).collect::<HashSet<(UVec2, UVec2)>>().len();

        println!("sum: {}", sum);
        assert_eq!(sum, 36);

        Ok(())
    }
}
