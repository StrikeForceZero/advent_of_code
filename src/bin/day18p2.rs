use num::traits::real::Real;
use pathfinding::prelude::astar;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use itertools::Itertools;
use advent_of_code::read_input;

// this is dirty but instead of rewriting Memory to not have to reparse everything each iteration
// we just ran it like this to get the answer
fn main() -> anyhow::Result<()> {
    let input = read_input(18)?;
    let mut ix = 0;
    let mut blocked_by = None;
    loop {
        ix += 1;
        println!("ix: {}", ix);
        let lines = input.lines().take(ix);
        let input = lines.clone().join("\n");
        let mut memory = Memory::try_from(input.as_str())?;
        memory.set_size(71);
        let path = memory.find_path()?;
        if path.is_none() {
            blocked_by = Some(lines.last().unwrap());
            break;
        }
    }
    println!("Blocked by: {:?}", blocked_by);
    Ok(())
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum MemoryCell {
    #[default]
    Safe,
    Corrupt,
}

impl Display for MemoryCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryCell::Safe => write!(f, "."),
            MemoryCell::Corrupt => write!(f, "#"),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
struct Memory(Vec<Vec<MemoryCell>>);

impl Memory {
    fn set_size(&mut self, size: usize) {
        for row in &mut self.0 {
            row.resize(size, MemoryCell::Safe);
        }
        self.0.resize(size, vec![MemoryCell::Safe; size]);
    }
    fn find_path(&self) -> anyhow::Result<Option<Vec<(usize, usize)>>> {
        let goal = (
            self.0[0].len().saturating_sub(1) as i32,
            self.0.len().saturating_sub(1) as i32,
        );
        // println!("goal: {goal:?}");
        if self.0[0][0] != MemoryCell::Safe || self.0[goal.1 as usize][goal.0 as usize] != MemoryCell::Safe {
            return Ok(None);
        }
        let result = astar(
            &(0i32, 0i32),
            |&(x, y)| {
                let neighbors = vec![
                    (x + 1, y),
                    (x - 1, y),
                    (x, y + 1),
                    (x, y - 1),
                ];
                // println!("Neighbors for ({x}, {y}): {neighbors:?}");
                neighbors
                    .into_iter()
                    .filter(|&(nx, ny)| {
                        let in_bounds = nx >= 0 && ny >= 0 && nx <= goal.0 && ny <= goal.1;
                        let is_safe = in_bounds && self.0[ny as usize][nx as usize] == MemoryCell::Safe;
                        if !is_safe {
                            // println!("Filtered out neighbor ({nx}, {ny}): in_bounds={in_bounds}, is_safe={is_safe}");
                        }
                        is_safe
                    })
                    .map(|pos| (pos, 1))
                    .collect::<Vec<_>>()
            },
            |&(x, y)| {
                /*println!(
                    "Heuristic for ({x}, {y}) -> ({gx}, {gy}): {}",

                    (goal.0 - x).abs() + (goal.1 - y).abs(),
                    gx = goal.0,
                    gy = goal.1,
                );*/
                (goal.0 - x).abs() + (goal.1 - y).abs()
            },
            |&p| p == goal,
        );
        //println!("Result: {:?}", result);
        let result = result.map(|(path, _cost)| {
            path.into_iter()
                .map(|pos| (pos.0 as usize, pos.1 as usize))
                .collect()
        });
        Ok(result)
    }
}

impl Display for Memory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for row in &self.0 {
            for cell in row {
                write!(f, "{}", cell)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl TryFrom<&str> for Memory {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        process_str(value)
    }
}

fn process_line(memory: &mut Memory, line: &str) -> anyhow::Result<()> {
    let mut parts = line.trim().split(',').map(str::parse::<usize>);
    let x = parts.next().expect("unexpected input")?;
    let y = parts.next().expect("unexpected input")?;
    assert!(parts.next().is_none());

    let width = (if memory.0.is_empty() {
        0
    } else {
        memory.0[0].len()
    })
    .max(x + 1);

    if memory.0.len() <= y {
        memory.0.resize(y + 1, vec![MemoryCell::Safe; width]);
    }

    if memory.0[0].len() <= width {
        for row in &mut memory.0 {
            row.resize(width, MemoryCell::Safe);
        }
    }
    // println!("{:?} {:?}", x, y);
    memory.0[y][x] = MemoryCell::Corrupt;
    Ok(())
}

fn process_lines(memory: &mut Memory, lines: &str) -> anyhow::Result<()> {
    for line in lines.lines() {
        process_line(memory, line)?;
    }
    Ok(())
}

fn process_str(input: &str) -> anyhow::Result<Memory> {
    let mut memory = Memory::default();
    process_lines(&mut memory, input)?;
    Ok(memory)
}

#[cfg(test)]
mod tests {
    use super::*;
    use advent_of_code::utils::string::deformat_string;
    use itertools::Itertools;
    #[test]
    fn examples() -> anyhow::Result<()> {
        let input = deformat_string(
            "
            5,4
            4,2
            4,5
            3,0
            2,1
            6,3
            2,4
            1,5
            0,6
            3,3
            2,6
            5,1
            1,2
            5,5
            2,5
            6,5
            1,4
            0,4
            6,4
            1,1
            6,1
            1,0
            0,5
            1,6
            2,0
        ",
        );
        let mut ix = 0;
        let mut blocked_by = None;
        loop {
            ix += 1;
            let lines = input.lines().take(ix);
            let input = lines.clone().join("\n");
            let mut memory = Memory::try_from(input.as_str())?;
            memory.set_size(7);
            let path = memory.find_path()?;
            if path.is_none() {
                blocked_by = Some(lines.last().unwrap());
                break;
            }
        }
        assert_eq!(blocked_by, Some("6,1"));
        Ok(())
    }
}
