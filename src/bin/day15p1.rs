use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use anyhow::anyhow;
use glam::{IVec2, UVec2};
use itertools::Itertools;
use advent_of_code::read_input;
use advent_of_code::utils::matrix::MatrixDetails;
use advent_of_code::utils::vec2::{IntoIVec2, IntoUsizeTuple, TryIntoUVec2};

fn main() -> anyhow::Result<()> {
    let input = read_input(15)?;
    let mut instance = parse_input(&input)?;
    let gps_sum = instance.process()?;
    println!("{}", gps_sum);
    Ok(())
}

fn split_board_and_inputs(input: &str) -> anyhow::Result<(String, String)> {
    let mut parts = input.split("\n\n").map(|line| line.to_string());
    let board = parts.next().ok_or(anyhow!("missing board"))?;
    let inputs = parts.next().ok_or(anyhow!("missing inputs"))?;
    if parts.next().is_some() {
        return Err(anyhow!("too many sections"));
    }
    Ok((board, inputs))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Piece {
    Wall,
    Robot,
    Box,
}

fn parse_board(board: &str) -> anyhow::Result<Vec<Vec<Option<Piece>>>> {
    board
        .lines()
        .map(|row| {
            row.chars()
                .map(|c| {
                    match c {
                        '#' => Ok(Some(Piece::Wall)),
                        '@' => Ok(Some(Piece::Robot)),
                        'O' => Ok(Some(Piece::Box)),
                        '.' => Ok(None),
                        _ => Err(anyhow::anyhow!("unexpected char {}", c)),
                    }
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn to_offset(&self) -> IVec2 {
        match self {
            Direction::Up => IVec2::NEG_Y,
            Direction::Down => IVec2::Y,
            Direction::Left => IVec2::NEG_X,
            Direction::Right => IVec2::X,
        }
    }
    fn next_pos(&self, pos: IVec2) -> IVec2 {
        pos + self.to_offset()
    }
}

impl TryFrom<char> for Direction {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        Ok(match value {
            '^' => Self::Up,
            'v' => Self::Down,
            '<' => Self::Left,
            '>' => Self::Right,
            _ => return Err(anyhow!("invalid direction {value}")),
        })
    }
}

fn parse_inputs(inputs: &str) -> anyhow::Result<Vec<Direction>> {
    inputs.chars().filter_map(|c| if c.is_whitespace() { None } else { Some(c.try_into()) }).collect()
}

fn parse_input(input: &str) -> anyhow::Result<Instance> {
    let (board, inputs) = split_board_and_inputs(input)?;
    let board = parse_board(&board)?;
    let inputs = parse_inputs(&inputs)?;
    Instance::new(board, inputs)
}

enum InstanceStepResult {
    Step,
    Done,
}

#[derive(Debug, Default)]
struct Instance {
    board: Vec<Vec<Option<Piece>>>,
    robot_pos: UVec2,
    inputs: VecDeque<Direction>,
    matrix_details: MatrixDetails,
}

impl Instance {
    fn new(board: Vec<Vec<Option<Piece>>>, inputs: Vec<Direction>) -> anyhow::Result<Self> {
        let mut robot_pos = None;
        for (y, row) in board.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                match cell {
                    Some(Piece::Robot) => {
                        if robot_pos.replace(UVec2::new(x as u32, y as u32)).is_some() {
                            return Err(anyhow!("duplicate robot"));
                        }
                    },
                    _ => {}
                }
            }
        }
        let Some(robot_pos) = robot_pos else { return Err(anyhow!("missing robot")) };
        let inputs = inputs.into();
        let matrix_details = MatrixDetails::from_matrix(&board);
        Ok(Self {
            board,
            robot_pos,
            inputs,
            matrix_details,
        })
    }
    fn step(&mut self) -> InstanceStepResult {
        let Some(direction) = self.inputs.pop_front() else { return InstanceStepResult::Done; };
        let mut next_pos = self.robot_pos.into_ivec2();
        let mut newtons_cradle = vec![self.robot_pos];
        while let Ok(pos) = direction.next_pos(next_pos).try_into_uvec2() {
            if !self.matrix_details.is_within_bounds(pos) {
                break;
            }
            next_pos = pos.into_ivec2();
            let Ok((x, y)) = next_pos.try_into_uvec2().map(|pos | pos.into_usize_tuple()) else { break; };
            let cell = &mut self.board[y][x];
            match cell {
                None => {
                    newtons_cradle.push(pos);
                    break;
                },
                Some(Piece::Wall) => {
                    // can't move, clear the cradle
                    newtons_cradle.clear();
                    break;
                },
                Some(Piece::Box) => newtons_cradle.push(pos),
                Some(Piece::Robot) => unreachable!(),
            }
        }
        let Some(robot_pos) = newtons_cradle.get(1) else { return InstanceStepResult::Step };
        self.robot_pos = *robot_pos;
        for (to, from) in newtons_cradle.into_iter().rev().tuple_windows() {
            let Some(from) = self.board[from.y as usize][from.x as usize].take() else { unreachable!() };
            if self.board[to.y as usize][to.x as usize].replace(from).is_some() {
                unreachable!();
            }
        }
        if self.inputs.is_empty() {
            InstanceStepResult::Done
        } else {
            InstanceStepResult::Step
        }
    }
    fn gps(&mut self) -> usize {
        let mut gps_sum = 0;
        for (y, row) in self.board.iter().enumerate() {
            for (x, col) in row.iter().enumerate() {
                match col {
                    None => {}
                    Some(Piece::Wall) => {}
                    Some(Piece::Robot) => {}
                    Some(Piece::Box) => gps_sum += 100 * y + x,
                }
            }
        }
        gps_sum
    }
    fn process(&mut self) -> anyhow::Result<u32> {
        loop {
            match self.step() {
                InstanceStepResult::Step => continue,
                InstanceStepResult::Done => return Ok(self.gps() as u32),
            }
        }
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for row in self.board.iter() {
            for col in row {
                match col {
                    None => write!(f, ".")?,
                    Some(Piece::Wall) => write!(f, "#")?,
                    Some(Piece::Robot) => write!(f, "@")?,
                    Some(Piece::Box) => write!(f, "O")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use advent_of_code::utils::string::deformat_string;
    use super::*;
    #[test]
    fn examples() -> anyhow::Result<()> {
        let input = deformat_string("
            ########
            #..O.O.#
            ##@.O..#
            #...O..#
            #.#.O..#
            #...O..#
            #......#
            ########

            <^^>>>vv<v>>v<<
        ");
        let mut instance = parse_input(&input)?;
        let gps_sum = instance.process()?;
        assert_eq!(gps_sum, 2028);

        let input = deformat_string("
            ##########
            #..O..O.O#
            #......O.#
            #.OO..O.O#
            #..O@..O.#
            #O#..O...#
            #O..O..O.#
            #.OO.O.OO#
            #....O...#
            ##########

            <vv>^<v^>v>^vv^v>v<>v^v<v<^vv<<<^><<><>>v<vvv<>^v^>^<<<><<v<<<v^vv^v>^
            vvv<<^>^v^^><<>>><>^<<><^vv^^<>vvv<>><^^v>^>vv<>v<<<<v<^v>^<^^>>>^<v<v
            ><>vv>v^v^<>><>>>><^^>vv>v<^^^>>v^v^<^^>v^^>v^<^v>v<>>v^v^<v>v^^<^^vv<
            <<v<^>>^^^^>>>v^<>vvv^><v<<<>^^^vv^<vvv>^>v<^^^^v<>^>vvvv><>>v^<<^^^^^
            ^><^><>>><>^^<<^^v>>><^<v>^<vv>>v>>>^v><>^v><<<<v>>v<v<v>vvv>^<><<>^><
            ^>><>^v<><^vvv<^^<><v<<<<<><^v<<<><<<^^<v<^^^><^>>^<v^><<<^>>^v<v^v<v^
            >^>>^v>vv>^<<^v<>><<><<v<<v><>v<^vv<<<>^^v^>^^>>><<^v>>v^v><^^>>^<>vv^
            <><^^>^^^<><vvvvv^v<v<<>^v<v>v<<^><<><<><<<^^<<<^<<>><<><^^^>^^<>^>v<>
            ^^>vv<^v^v<vv>^<><v<^v>^^^>>>^^vvv^>vvv<>>>^<^>>>>>^<<^v>^vvv<>^<><<v>
            v^^>>><<^^<>>^v^<v^vv<>v^<<>^<^v^v><^<<<><<^<v><v<>vv>>v><v^<vv<>v^<<^
        ");
        let mut instance = parse_input(&input)?;
        let gps_sum = instance.process()?;
        assert_eq!(gps_sum, 10092);
        Ok(())
    }
}
