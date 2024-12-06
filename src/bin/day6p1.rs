use std::collections::HashSet;
use advent_of_code::read_input;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let board = read_input(6).map(|line| line.expect("failed to read input"));
    let board = board.map(|line| line.chars().collect::<Vec<_>>()).collect::<Vec<_>>();
    let Some(mut walker) = GuardWalker::init(&board) else {
        panic!("failed to find start pos");
    };
    loop {
        if walker.move_forward(&board).is_none() {
            break;
        }
    }
    println!("Answer: {}", walker.seen.len());
    Ok(())
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
struct Pos {
    x: usize,
    y: usize,
}

impl Pos {
    fn new(x: usize, y: usize) -> Self {
        Self {
            x,
            y,
        }
    }
}

impl From<(usize, usize)> for Pos {
    fn from((x, y): (usize, usize)) -> Self {
        Self { x, y }
    }
}

type Board = Vec<Vec<char>>;


#[derive(Debug, Clone, Copy, Default, PartialEq, Hash)]
enum Direction {
    #[default]
    Up,
    Down,
    Left,
    Right
}

impl Direction {
    fn turn(&mut self) {
        *self = match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        };
    }
    fn next(&self, pos: Pos, min_pos: Pos, max_pos: Pos) -> Option<Pos> {
        let Pos { x, y} = pos;
        let (delta_x, delta_y): (i32, i32) = match self {
            Self::Up => (0, -1),
            Self::Right => (1, 0),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
        };
        if
            x == min_pos.x && delta_x < 0
            || y == min_pos.y && delta_y < 0
            || x == max_pos.x && delta_x > 0
            || y == max_pos.y && delta_y > 0
        {
            return None;
        }
        let next_x = (x as i32 + delta_x) as usize;
        let next_y = ( y as i32 + delta_y) as usize;
        Some(Pos::new(next_x, next_y))
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct GuardWalker {
    seen: HashSet<Pos>,
    current_pos: Pos,
    direction: Direction,
}

impl GuardWalker {
    fn new(pos: Pos) -> Self {
        Self {
            seen: HashSet::from([pos]),
            current_pos: pos,
            ..Default::default()
        }
    }
    fn move_forward(&mut self, board: &Board) -> Option<Pos> {
        let width = board[0].len();
        let height = board.len();
        let Some(next_pos) = self.direction.next(self.current_pos, Pos::default(), Pos::new(width - 1, height - 1)) else {
            return None;
        };
        if next_pos.x >= board[0].len() || next_pos.y >= board.len() {
            return None;
        } else if board[next_pos.y][next_pos.x] == '#' {
            self.direction.turn();
        } else {
            self.current_pos = next_pos;
            self.seen.insert(self.current_pos);
        }
        Some(self.current_pos)
    }
    fn init(board: &Board) -> Option<Self> {
        let width = board[0].len();
        let height = board.len();
        for y in 0..height {
            for x in 0..width {
                if board[y][x] == '^' {
                    return Some(Self::new(Pos::new(x, y)));
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{Direction, Pos};

    #[test]
    fn direction_turn_test() {
        let mut dir = Direction::Up;
        dir.turn();
        assert_eq!(dir, Direction::Right);
    }

    #[test]
    fn direction_next_test() {
        let mut dir = Direction::Down;
        assert_eq!(dir.next(Pos::default(), Pos::default(), Pos::new(1,1)), Some(Pos::new(0,1)));
        assert_eq!(dir.next(Pos::default(), Pos::default(), Pos::default()), None);
    }
}
