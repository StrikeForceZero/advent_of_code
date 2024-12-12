use advent_of_code::read_input;
use advent_of_code::utils::matrix::{MatrixDetails, MatrixIterator};
use advent_of_code::utils::string::StringToCharsMatrix;
use advent_of_code::utils::vec2::IntoIVec2;
use glam::{IVec2, UVec2};
use itertools::Itertools;
use std::collections::{HashMap, HashSet, VecDeque};
use std::hash::{Hash, Hasher};
use std::num::TryFromIntError;
use thiserror::Error;

fn main() -> anyhow::Result<()> {
    let garden = read_input(12)?.to_chars_matrix();
    let pas = calc_perimeter_areas(&garden);
    let cost: usize = pas.iter().map(|pa| pa.perimeter_area.cost()).sum();
    println!("answer: {cost}");
    Ok(())
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PerimeterArea {
    sides: usize,
    perimeter: usize,
    area: usize,
}

impl PerimeterArea {
    fn cost(self) -> usize {
        self.sides * self.area
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Axis {
    #[default]
    X,
    Y,
}

impl From<Direction> for Axis {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Up => Self::Y,
            Direction::Right => Self::X,
            Direction::Down => Self::Y,
            Direction::Left => Self::X,
        }
    }
}

const DIRECTIONS: [Direction; 4] = [
    Direction::Up,
    Direction::Right,
    Direction::Down,
    Direction::Left,
];
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Direction {
    #[default]
    Up,
    Right,
    Down,
    Left,
}
impl Direction {
    fn all() -> impl Iterator<Item = Self> {
        DIRECTIONS.into_iter()
    }
    fn rotate(self) -> Self {
        match self {
            Self::Up => Self::Right,
            Self::Right => Self::Down,
            Self::Down => Self::Left,
            Self::Left => Self::Up,
        }
    }
    fn inverse(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
            Self::Left => Self::Right,
        }
    }
    fn next_uvec2(self, pos: UVec2) -> Result<UVec2, TryFromIntError> {
        let next_pos = self.next_ivec2(pos.as_ivec2());
        UVec2::try_from(next_pos)
    }
    fn next_ivec2(self, pos: IVec2) -> IVec2 {
        IVec2::from(self) + pos
    }
}

impl From<Direction> for IVec2 {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::Up => Self::NEG_Y,
            Direction::Down => Self::Y,
            Direction::Left => Self::NEG_X,
            Direction::Right => Self::X,
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
            IVec2::NEG_Y => Self::Up,
            IVec2::Y => Self::Down,
            IVec2::NEG_X => Self::Left,
            IVec2::X => Self::Right,
            _ => return Err(UnknownDirection),
        })
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Ord, PartialOrd, Eq, Hash)]
struct GardenBed {
    label: char,
    perimeter_area: PerimeterArea,
}

fn boundry_points(points: HashMap<UVec2, HashSet<Direction>>) -> HashMap<UVec2, HashSet<Direction>> {
    points
        .into_iter()
        .filter_map(|(pos, connections)| {
            if DIRECTIONS
                .iter()
                .all(|direction| connections.contains(direction))
            {
                None
            } else {
                Some((pos, connections))
            }
        })
        .collect()
}

#[derive(Debug, Copy, Clone, Default, Eq)]
struct Edge {
    start: IVec2,
    end: IVec2,
    axis: Axis,
}

impl PartialEq for Edge {
    fn eq(&self, other: &Self) -> bool {
        self.axis == other.axis && (self.start == other.start && self.end == other.end || self.start == other.end && self.end == other.start)
    }
}

impl Hash for Edge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let is_start_smaller = self.start.x < self.end.x || (self.start.x == self.end.x && self.start.y < self.end.y);
        let (a, b) = if is_start_smaller {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        };
        a.hash(state);
        b.hash(state);
        self.axis.hash(state)
    }
}


fn calc_edges(points: &HashMap<UVec2, HashSet<Direction>>) -> Vec<Edge> {
    let points = points
        .iter()
        .map(|(pos, directions)| (pos.as_ivec2(), directions))
        .collect::<HashMap<_, _>>();
    let mut edges = vec![];
    for (pos, connections) in points {
        for direction in DIRECTIONS.iter() {
            if connections.contains(direction) {
                let next_pos = direction.next_ivec2(pos);
                edges.push(Edge {
                    start: pos,
                    end: next_pos,
                    axis: Axis::from(*direction),
                });
            }
        }
    }
    edges
}

fn calculate_sides(edges: Vec<Edge>) -> usize {
    let mut visited = HashSet::new();
    let mut sides = 0;

    for edge in &edges {
        if visited.contains(edge) {
            continue;
        }

        // Start flood-fill for continuous edges
        let mut queue = VecDeque::new();
        queue.push_back(*edge);
        visited.insert(*edge);

        while let Some(current) = queue.pop_front() {
            for neighbor in find_contiguous_edges(&current, &edges) {
                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                }
            }
        }

        // Count a side after processing a contiguous group
        sides += 1;
    }

    sides
}

fn find_contiguous_edges(edge: &Edge, edges: &[Edge]) -> Vec<Edge> {
    edges
        .iter()
        .filter(|&e| {
            // Check if the edges are connected and on same axis
            e.axis == edge.axis && are_edges_connected(edge, e)
        })
        .copied()
        .collect()
}

fn are_edges_connected(e1: &Edge, e2: &Edge) -> bool {
    e1.end == e2.start || e1.start == e2.end || e1.start == e2.start || e1.end == e2.end
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

        let mut points: HashMap<UVec2, HashSet<Direction>> = Default::default();
        let mut pa = PerimeterArea::default();

        let next_pos = |pos: UVec2, direction: Direction| -> Option<UVec2> {
            direction.next_uvec2(pos).ok().and_then(|next_pos| {
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

            points.entry(cur_pos).or_default();
            pa.area += 1;

            'direction: for direction in Direction::all() {
                let Some(next_pos) = next_pos(cur_pos, direction) else {
                    pa.perimeter += 1;
                    continue 'direction;
                };
                points.entry(cur_pos).or_default().insert(direction);
                if seen.contains(&next_pos) {
                    continue 'direction;
                }
                search_queue.push_back(next_pos);
            }
        }
        let boundry_points = boundry_points(points);
        let edges = calc_edges(&boundry_points);
        let sides = calculate_sides(edges);
        pa.sides = sides;
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
    fn test_boundry_points() {
        let mut points = HashMap::new();
        points.insert(
            UVec2 { x: 0, y: 0 },
            HashSet::from([Direction::Right, Direction::Down]),
        );
        points.insert(
            UVec2 { x: 1, y: 0 },
            HashSet::from([Direction::Left, Direction::Down]),
        );
        points.insert(
            UVec2 { x: 0, y: 1 },
            HashSet::from([Direction::Right, Direction::Up]),
        );
        points.insert(
            UVec2 { x: 1, y: 1 },
            HashSet::from([Direction::Left, Direction::Up]),
        );

        let boundary = boundry_points(points);

        // Check that boundary points only include edges, not fully surrounded points.
        assert_eq!(boundary.len(), 4);
        assert!(boundary.contains_key(&UVec2 { x: 0, y: 0 }));
        assert!(boundary.contains_key(&UVec2 { x: 1, y: 0 }));
        assert!(boundary.contains_key(&UVec2 { x: 0, y: 1 }));
        assert!(boundary.contains_key(&UVec2 { x: 1, y: 1 }));
    }

    #[test]
    fn test_calc_edges() {
        let mut points = HashMap::new();
        points.insert(
            UVec2 { x: 0, y: 0 },
            HashSet::from([Direction::Right, Direction::Down]),
        );
        points.insert(
            UVec2 { x: 1, y: 0 },
            HashSet::from([Direction::Left, Direction::Down]),
        );

        let edges = calc_edges(&points);

        // Check that edges are generated correctly.
        assert_eq!(edges.len(), 4);
        assert!(edges.contains(&Edge {
            start: IVec2 { x: 0, y: 0 },
            end: IVec2 { x: 1, y: 0 },
            axis: Axis::X,
        }));
        assert!(edges.contains(&Edge {
            start: IVec2 { x: 0, y: 0 },
            end: IVec2 { x: 0, y: 1 },
            axis: Axis::Y,
        }));
    }

    #[test]
    fn test_calculate_sides() {
        let edges = vec![
            Edge {
                start: IVec2 { x: 0, y: 0 },
                end: IVec2 { x: 1, y: 0 },
                axis: Axis::X,
            },
            Edge {
                start: IVec2 { x: 1, y: 0 },
                end: IVec2 { x: 2, y: 0 },
                axis: Axis::X,
            },
            Edge {
                start: IVec2 { x: 0, y: 0 },
                end: IVec2 { x: 0, y: 1 },
                axis: Axis::X,
            },
            Edge {
                start: IVec2 { x: 0, y: 1 },
                end: IVec2 { x: 0, y: 2 },
                axis: Axis::X,
            },
            Edge {
                start: IVec2 { x: 0, y: 2 },
                end: IVec2 { x: 0, y: 3 },
                axis: Axis::Y,
            },
        ];

        let sides = calculate_sides(edges);

        // Check that contiguous edges are grouped into sides.
        assert_eq!(sides, 2);
    }

    #[test]
    fn test_find_neighbors() {
        let edges = vec![
            Edge {
                start: IVec2 { x: 0, y: 0 },
                end: IVec2 { x: 1, y: 0 },
                axis: Axis::X,
            },
            Edge {
                start: IVec2 { x: 1, y: 0 },
                end: IVec2 { x: 2, y: 0 },
                axis: Axis::X,
            },
            Edge {
                start: IVec2 { x: 0, y: 0 },
                end: IVec2 { x: 0, y: 1 },
                axis: Axis::Y,
            },
        ];

        let neighbors = find_contiguous_edges(
            &Edge {
                start: IVec2 { x: 0, y: 0 },
                end: IVec2 { x: 1, y: 0 },
                axis: Axis::X,
            },
            &edges,
        );

        // Check that neighbors include the correct contiguous edges.
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&Edge {
            start: IVec2 { x: 1, y: 0 },
            end: IVec2 { x: 2, y: 0 },
            axis: Axis::X,
        }));
    }



    #[test]
    fn calc_perimeter_areas_test() {
        let matrix = deformat_string(
            "
                AA
                AA
            ",
        )
        .to_chars_matrix();
        let pas = calc_perimeter_areas(&matrix);
        assert_eq!(
            pas,
            vec![GardenBed {
                label: 'A',
                perimeter_area: PerimeterArea {
                    area: 4,
                    sides: 4,
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
                        sides: 6,
                        perimeter: 8
                    },
                },
                GardenBed {
                    label: 'B',
                    perimeter_area: PerimeterArea {
                        area: 1,
                        sides: 4,
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
                    sides: 10,
                    perimeter: 18,
                },
            },
            GardenBed {
                label: 'I',
                perimeter_area: PerimeterArea {
                    area: 4,
                    sides: 4,
                    perimeter: 8,
                },
            },
            GardenBed {
                label: 'C',
                perimeter_area: PerimeterArea {
                    area: 14,
                    sides: 22,
                    perimeter: 28,
                },
            },
            GardenBed {
                label: 'F',
                perimeter_area: PerimeterArea {
                    area: 10,
                    sides: 12,
                    perimeter: 18,
                },
            },
            GardenBed {
                label: 'V',
                perimeter_area: PerimeterArea {
                    area: 13,
                    sides: 10,
                    perimeter: 20,
                },
            },
            GardenBed {
                label: 'J',
                perimeter_area: PerimeterArea {
                    area: 11,
                    sides: 12,
                    perimeter: 20,
                },
            },
            GardenBed {
                label: 'C',
                perimeter_area: PerimeterArea {
                    area: 1,
                    sides: 4,
                    perimeter: 4,
                },
            },
            GardenBed {
                label: 'E',
                perimeter_area: PerimeterArea {
                    area: 13,
                    sides: 8,
                    perimeter: 18,
                },
            },
            GardenBed {
                label: 'I',
                perimeter_area: PerimeterArea {
                    area: 14,
                    sides: 16,
                    perimeter: 22,
                },
            },
            GardenBed {
                label: 'M',
                perimeter_area: PerimeterArea {
                    area: 5,
                    sides: 6,
                    perimeter: 12,
                },
            },
            GardenBed {
                label: 'S',
                perimeter_area: PerimeterArea {
                    area: 3,
                    sides: 6,
                    perimeter: 8,
                },
            },
        ]
        .into_iter()
        .sorted()
        .collect::<Vec<_>>();
        assert_eq!(perimeter_areas, expected,);
        assert_eq!(
            perimeter_areas
                .iter()
                .map(|pa| pa.perimeter_area.cost())
                .sum::<usize>(),
            1206,
        );
        Ok(())
    }
}
