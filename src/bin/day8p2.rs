use std::collections::{HashMap, HashSet};
use std::ops::{Add, Neg, Sub};
use itertools::Itertools;
use advent_of_code::read_input;
use advent_of_code::utils::string::deformat_string;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Pos<T> {
    x: T,
    y: T,
}

impl<T> Pos<T> {
    fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
        }
    }
}

impl<T> Neg for Pos<T> where T: Neg<Output = T> {
    type Output = Pos<T>;

    fn neg(self) -> Self::Output {
        Pos::new(-self.x, -self.y)
    }
}

impl<T> Sub for Pos<T> where T: Sub<Output = T> {
    type Output = Pos<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Pos::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T> Add for Pos<T> where T: Add<Output = T> {
    type Output = Pos<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Pos::new(self.x + rhs.x, self.y + rhs.y)
    }
}


impl<T> From<(T, T)> for Pos<T> {
    fn from(value: (T, T)) -> Self {
        Pos::new(value.0, value.1)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
struct Marker {
    antenna: Option<char>,
    anti_node: bool,
}

impl Marker {
    fn antenna(char: char) -> Self {
        Self {
            antenna: Some(char),
            anti_node: false,
        }
    }
    fn mark_anti_node(&mut self) {
        self.anti_node = true;
    }
}

type Data = Vec<Vec<Marker>>;
type SeenMap = HashMap<char, Vec<Pos<i32>>>;

fn insert_anti_node(data: &mut Data, pos: Pos<i32>, delta: Pos<i32>) {
    let mut pos = pos + delta;
    while pos.x >= 0 && pos.y >= 0 && pos.x < data[0].len() as i32 && pos.y < data.len() as i32 {
        data[pos.y as usize][pos.x as usize].mark_anti_node();
        pos = pos + delta;
    }
}


fn process_seen(data: &mut Data, seen: SeenMap) {
    for (_, positions) in seen.into_iter() {
        if positions.len() < 2 {
            continue;
        }
        for (&a, &b) in positions.iter().tuple_combinations() {
            let delta = a - b;
            insert_anti_node(data, a, delta);
            insert_anti_node(data, b, -delta);
        }
    }
}

fn mark_seen(data: &Data, pos: Pos<i32>, seen: &mut SeenMap) {
    if let Some(char) = data[pos.y as usize][pos.x as usize].antenna {
        seen.entry(char).or_default().push(pos);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lines = read_input(8).map(|line| line.expect("failed to read input"));
    // et lines = deformat_string("\
    //    ............
    //    ........0...
    //    .....0......
    //    .......0....
    //    ....0.......
    //    ......A.....
    //    ............
    //    ............
    //    ........A...
    //    .........A..
    //    ............
    //    ............
    // );
    // et lines = lines.lines();
    let mut data: Data = vec![];
    for line in lines {
        let mut row = vec![];
        for char in line.chars() {
            let col = if char == '.' {
                Marker::default()
            } else {
                Marker::antenna(char)
            };
            row.push(col);
        }
        data.push(row);
    }

    let height = data.len() as i32;
    let width = {
        let row_lens = data.iter().map(|row| row.len());
        let width = row_lens.clone().max().unwrap_or_default();
        if width == 0 || row_lens.min().unwrap_or_default() != width {
            panic!("missing columns");
        }
        width as i32
    };

    let mut seen = SeenMap::new();
    // row/col
    for y in 0..height {
        for x in 0..width {
            mark_seen(&data, Pos::new(x, y), &mut seen);
        }
    }
    process_seen(&mut data, seen);

    let mut anti_node_count = 0;
    for row in data.into_iter() {
        for col in row.into_iter() {
            if let Some(char) = col.antenna {
                anti_node_count += 1;
                print!("{char}");
            } else if col.anti_node {
                anti_node_count += 1;
                print!("#");
            } else {
                print!(".");
            }
        }
        println!();
    }

    println!("Answer: {}", anti_node_count);

    Ok(())
}
