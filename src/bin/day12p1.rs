use advent_of_code::utils::matrix::{MatrixDetails, MatrixIterator};
use advent_of_code::utils::vec2::IntoIVec2;
use glam::{IVec2, UVec2};
use std::collections::{HashMap, HashSet, VecDeque};
use std::num::TryFromIntError;
use thiserror::Error;
use advent_of_code::read_input;
use advent_of_code::utils::string::StringToCharsMatrix;

fn main() -> anyhow::Result<()> {
    let garden = read_input(12)?.to_chars_matrix();
    let pas = calc_perimeter_areas(&garden);
    let cost: usize = pas.iter().map(|pa| pa.perimeter_area.cost()).sum();
    println!("answer: {cost}");
    Ok(())
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PerimeterArea {
    perimeter: usize,
    area: usize,
}

impl PerimeterArea {
    fn cost(self) -> usize {
        self.perimeter * self.area
    }
}

const DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Direction {
    #[default]
    North,
    East,
    South,
    West,
}
impl Direction {
    fn all() -> impl Iterator<Item = Self> {
        DIRECTIONS.into_iter()
    }
    fn rotate(self) -> Self {
        match self {
            Self::North => Self::East,
            Self::East => Self::South,
            Self::South => Self::West,
            Self::West => Self::North,
        }
    }
    fn inverse(self) -> Self {
        match self {
            Self::North => Self::South,
            Self::East => Self::West,
            Self::South => Self::North,
            Self::West => Self::East,
        }
    }
    fn next(self, pos: UVec2) -> Result<UVec2, TryFromIntError> {
        let next_pos = IVec2::try_from(pos)? + IVec2::from(self);
        UVec2::try_from(next_pos)
    }
}

impl From<Direction> for IVec2 {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::North => Self::NEG_Y,
            Direction::South => Self::Y,
            Direction::West => Self::NEG_X,
            Direction::East => Self::X,
        }
    }
}

#[derive(Error, Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[error("unknown direction")]
struct UnknownDirection;

impl TryFrom<IVec2> for Direction {
    type Error = UnknownDirection;
    fn try_from(delta: IVec2) -> Result<Self, Self::Error> {
        Ok(match delta {
            IVec2::NEG_Y => Self::North,
            IVec2::Y => Self::South,
            IVec2::NEG_X => Self::West,
            IVec2::X => Self::East,
            _ => return Err(UnknownDirection),
        })
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Ord, PartialOrd, Eq, Hash)]
struct GardenBed {
    label: char,
    perimeter_area: PerimeterArea,
}

fn calc_perimeter_areas(garden: &Vec<Vec<char>>) -> Vec<GardenBed> {
    let matrix_details = MatrixDetails::from_matrix(garden);

    let mut results = vec![];
    let mut seen = HashSet::new();
    'matrix: for cur in MatrixIterator::new(garden) {
        let (cur_pos, &cur_char) = cur;

        if seen.contains(&cur_pos) {
            continue 'matrix;
        }

        let mut pa = PerimeterArea::default();

        let next_pos = |pos: UVec2, direction: Direction| -> Option<UVec2> {
            direction.next(pos).ok().and_then(|next_pos| {
                if matrix_details.is_within_bounds(next_pos) {
                    let next_char = garden[next_pos.y as usize][next_pos.x as usize];
                    if next_char == cur_char {
                        return Some(next_pos);
                    }
                }
                None
            })
        };

        let mut search_queue = VecDeque::from([cur_pos]);
        'search: while let Some(cur_pos) = search_queue.pop_front() {
            if seen.contains(&cur_pos) {
                continue 'search;
            }
            seen.insert(cur_pos);

            pa.area += 1;

            'direction: for direction in Direction::all() {
                let Some(next_pos) = next_pos(cur_pos, direction) else {
                    pa.perimeter += 1;
                    continue 'direction;
                };
                if seen.contains(&next_pos) {
                    continue 'direction;
                }
                search_queue.push_front(next_pos);
            }
        }

        results.push(GardenBed {
            label: cur_char,
            perimeter_area: pa,
        });
        seen.insert(cur_pos);
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use advent_of_code::utils::string::{deformat_string, StringToCharsMatrix};
    use itertools::Itertools;

    #[test]
    fn calc_perimeter_areas_test() {
        let matrix = deformat_string(
            "
                AA
                AA
            ",
        ).to_chars_matrix();
        let pas = calc_perimeter_areas(&matrix);
        assert_eq!(
            pas,
            vec![GardenBed {
                label: 'A',
                perimeter_area: PerimeterArea {
                    area: 4,
                    perimeter: 8
                },
            },],
        );

        let matrix = deformat_string(
            "
                AB
                AA
            ",
        )
        .to_chars_matrix();
        let pas = calc_perimeter_areas(&matrix);
        assert_eq!(
            pas,
            vec![
                GardenBed {
                    label: 'A',
                    perimeter_area: PerimeterArea {
                        area: 3,
                        perimeter: 8
                    },
                },
                GardenBed {
                    label: 'B',
                    perimeter_area: PerimeterArea {
                        area: 1,
                        perimeter: 4
                    },
                },
            ],
        )
    }
    #[test]
    fn examples() -> anyhow::Result<()> {
        let garden = deformat_string(
            "
                RRRRIICCFF
                RRRRIICCCF
                VVRRRCCFFF
                VVRCCCJFFF
                VVVVCJJCFE
                VVIVCCJJEE
                VVIIICJJEE
                MIIIIIJJEE
                MIIISIJEEE
                MMMISSJEEE
            ",
        )
        .to_chars_matrix();
        let perimeter_areas = calc_perimeter_areas(&garden)
            .into_iter()
            .sorted()
            .collect::<Vec<_>>();
        let expected = vec![
            GardenBed {
                label: 'R',
                perimeter_area: PerimeterArea {
                    area: 12,
                    perimeter: 18,
                },
            },
            GardenBed {
                label: 'I',
                perimeter_area: PerimeterArea {
                    area: 4,
                    perimeter: 8,
                },
            },
            GardenBed {
                label: 'C',
                perimeter_area: PerimeterArea {
                    area: 14,
                    perimeter: 28,
                },
            },
            GardenBed {
                label: 'F',
                perimeter_area: PerimeterArea {
                    area: 10,
                    perimeter: 18,
                },
            },
            GardenBed {
                label: 'V',
                perimeter_area: PerimeterArea {
                    area: 13,
                    perimeter: 20,
                },
            },
            GardenBed {
                label: 'J',
                perimeter_area: PerimeterArea {
                    area: 11,
                    perimeter: 20,
                },
            },
            GardenBed {
                label: 'C',
                perimeter_area: PerimeterArea {
                    area: 1,
                    perimeter: 4,
                },
            },
            GardenBed {
                label: 'E',
                perimeter_area: PerimeterArea {
                    area: 13,
                    perimeter: 18,
                },
            },
            GardenBed {
                label: 'I',
                perimeter_area: PerimeterArea {
                    area: 14,
                    perimeter: 22,
                },
            },
            GardenBed {
                label: 'M',
                perimeter_area: PerimeterArea {
                    area: 5,
                    perimeter: 12,
                },
            },
            GardenBed {
                label: 'S',
                perimeter_area: PerimeterArea {
                    area: 3,
                    perimeter: 8,
                },
            },
        ]
        .into_iter()
        .sorted()
        .collect::<Vec<_>>();
        assert_eq!(
            perimeter_areas,
            expected,
        );
        Ok(())
    }
}
