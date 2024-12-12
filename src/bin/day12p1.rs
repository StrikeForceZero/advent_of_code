use advent_of_code::utils::matrix::{MatrixDetails, MatrixIterator};
use advent_of_code::utils::vec2::IntoIVec2;
use glam::{IVec2, UVec2};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::num::TryFromIntError;
use thiserror::Error;

fn main() -> anyhow::Result<()> {
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct CacheEntry<T> {
    item: T,
    surrounding: HashMap<Direction, Option<char>>,
}

impl<T> CacheEntry<T>
where
    T: Default,
{
    fn new(item: T) -> Self {
        Self {
            item,
            ..Default::default()
        }
    }
}

fn calc_perimeter_areas(garden: &Vec<Vec<char>>) -> HashMap<char, PerimeterArea> {
    let matrix_details = MatrixDetails::from_matrix(garden);
    let mut perimeter_areas = HashMap::<char, PerimeterArea>::new();
    let mut cache = HashMap::<UVec2, CacheEntry<char>>::new();
    for cur in MatrixIterator::new(garden) {
        let (cur_pos, cur_char) = cur;
        cache
            .entry(cur_pos)
            .or_insert_with(|| CacheEntry::new(*cur_char));
        for direction in Direction::all() {
            let Some(cache_entry) = cache.get(&cur_pos) else { unreachable!() };
            if cache_entry.surrounding.contains_key(&direction) {
                continue;
            }

            let next: Option<(UVec2, &char)> = direction.next(cur_pos).ok().and_then(|next_pos| {
                // get next char
                if matrix_details.is_within_bounds(next_pos) {
                    let next_char = &garden[next_pos.y as usize][next_pos.x as usize];

                    // add inverse
                    let inverse_cache_entry = cache.entry(next_pos).or_insert_with(|| CacheEntry::new(*next_char));
                    inverse_cache_entry
                        .surrounding
                        .entry(direction.inverse())
                        .or_insert(Some(*cur_char));

                    Some((next_pos, next_char))
                } else {
                    None
                }
            });

            let next_char = next.map(|(_, next_char)| *next_char);
            let Some(cache_entry) = cache.get_mut(&cur_pos) else {
                unreachable!()
            };
            cache_entry.surrounding.insert(direction, next_char);
        }
    }
    for (_pos, cache_entry) in cache {
        let pa = perimeter_areas.entry(cache_entry.item).or_default();
        pa.area += 1;
        for (_, next_char) in cache_entry.surrounding {
            if Some(cache_entry.item) != next_char {
                pa.perimeter += 1;
            }
        }
    }
    perimeter_areas
}

#[cfg(test)]
mod tests {
    use super::*;
    use advent_of_code::utils::string::{deformat_string, StringToCharsMatrix};
    use std::collections::HashSet;
    #[test]
    fn calc_perimeter_areas_test() {
        let matrix = vec![vec!['A';2];2];
        let pas = calc_perimeter_areas(&matrix);
        assert_eq!(pas, HashMap::from([('A', PerimeterArea { area: 4, perimeter: 8 })]));

        let matrix = deformat_string(
            "
                AB
                AA
            ",
        )
            .to_chars_matrix();
        let pas = calc_perimeter_areas(&matrix);
        assert_eq!(pas, HashMap::from([
            ('A', PerimeterArea { area: 3, perimeter: 8 }),
            ('B', PerimeterArea { area: 1, perimeter: 4 }),
        ]))
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
        let perimeter_areas = calc_perimeter_areas(&garden);
        let expected = HashMap::from([
            (
                'R',
                PerimeterArea {
                    area: 12,
                    perimeter: 18,
                },
            ), // 216
            (
                'I',
                PerimeterArea {
                    area: 4,
                    perimeter: 8,
                },
            ), // 32
            (
                'C',
                PerimeterArea {
                    area: 14,
                    perimeter: 28,
                },
            ), // 392
            (
                'F',
                PerimeterArea {
                    area: 10,
                    perimeter: 18,
                },
            ), // 180
            (
                'V',
                PerimeterArea {
                    area: 13,
                    perimeter: 20,
                },
            ), // 260
            (
                'J',
                PerimeterArea {
                    area: 11,
                    perimeter: 20,
                },
            ), // 220
            (
                'C',
                PerimeterArea {
                    area: 1,
                    perimeter: 4,
                },
            ), // 4
            (
                'E',
                PerimeterArea {
                    area: 13,
                    perimeter: 18,
                },
            ), // 234
            (
                'I',
                PerimeterArea {
                    area: 14,
                    perimeter: 22,
                },
            ), // 308
            (
                'M',
                PerimeterArea {
                    area: 5,
                    perimeter: 12,
                },
            ), // 60
            (
                'S',
                PerimeterArea {
                    area: 3,
                    perimeter: 8,
                },
            ), // 24
        ]);
        assert_eq!(
            perimeter_areas.len(), expected.len(),
            "incorrect number of PerimeterAreas"
        );
        assert_eq!(
            perimeter_areas.keys().collect::<HashSet<_>>(),
            expected.keys().collect::<HashSet<_>>(),
            "incorrect PerimeterArea keys"
        );
        for (char, perimeter_area) in perimeter_areas {
            println!("{char}: {}", perimeter_area.cost());
            assert_eq!(
                perimeter_area, expected[&char],
                "incorrect PerimeterArea for {char}"
            );
        }
        Ok(())
    }
}
