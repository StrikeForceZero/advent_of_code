use std::error::Error;
use std::num::TryFromIntError;
use thiserror::Error;

#[derive(Debug, Clone)]
struct Report {
    data: Vec<u32>,
    direction: Option<Direction>,
    indexed_errors: Vec<IsNotSafeError>,
}

impl Report {
    fn from_data(data: Vec<u32>) -> Self {
        let (direction, indexed_errors) = validation_details(&data);
        Self {
            data,
            direction,
            indexed_errors,
        }
    }
    fn is_safe(&self) -> bool {
        self.indexed_errors.is_empty()
    }
    fn remove_bad_item_cloned(&self, invalid_ix: usize) -> Self {
        let mut new_data = self.data.clone();
        if invalid_ix < new_data.len() {
            new_data.remove(invalid_ix);
        }
        Self::from_data(new_data)
    }

    fn fix(self) -> Self {
        self.fix_cloned().map(|(_, report)| report).unwrap_or(self)
    }

    /// Attempts to smartly remove any single bad items to pass validation
    /// returns Some((true, Self)) if it had to fix the data
    /// returns Some((false, Self)) if it did not have to fix the data
    /// returns None if it failed to fix the data
    fn fix_cloned(&self) -> Option<(bool, Self)> {
        if self.is_safe() {
            return Some((false, self.clone()));
        }

        let invalid_distances = self.collect_indices(IsNotSafeError::InvalidDistance);
        if is_single_or_adjacent(&invalid_distances) {
            if let Some(fixed) = self.try_fix_invalid_indices(&invalid_distances) {
                return Some(fixed);
            }
        }

        let mixed_directions = self.collect_indices(IsNotSafeError::MixedDirection);
        if is_single_or_adjacent(&mixed_directions) {
            if let Some(fixed) = self.try_fix_invalid_indices(&mixed_directions) {
                return Some(fixed);
            }
        }

        /// similar to [`is_single_or_adjacent`], we are checking if there are 1-2 non error indices.
        // this attempts to remove one of the 2 "good" ones to see if it will then validate
        if [self.data.len() - 1, self.data.len() - 2].contains(&mixed_directions.len()) {
            for ix in 0..self.data.len() {
                if !self.contains_error_at(ix) {
                    let fixed = self.remove_bad_item_cloned(ix);
                    if fixed.is_safe() {
                        return Some((true, fixed));
                    }
                }
            }
        }

        None
    }

    fn collect_indices<F>(&self, matches: F) -> Vec<usize> where
        F: Fn(&IsNotSafeError) -> bool,
    {
        self.indexed_errors.iter().filter_map(|error| {
            if matches(error) {
                Some(error.index())
            } else {
                None
            }
        }).collect()
    }

    fn try_fix_invalid_indices(&self, indices: &[usize]) -> Option<(bool, Self)> {
        for &invalid_ix in indices.iter() {
            if let Some(fixed) = self.attempt_fix(invalid_ix) {
                return Some(fixed);
            }
        }
        None
    }

    fn attempt_fix(&self, index: usize) -> Option<(bool, Self)> {
        for &ix_offset in &[0, 1] {
            if index >= ix_offset {
                let fixed = self.remove_bad_item_cloned(index - ix_offset);
                if fixed.is_safe() {
                    return Some((true, fixed));
                }
            }
        }
        None
    }

    fn contains_error_at(&self, ix: usize) -> bool {
        self.indexed_errors.iter().any(|error| {
            matches!(error, IsNotSafeError::InvalidDistance { index, .. } | IsNotSafeError::MixedDirection { index, .. } if *index == ix)
        })
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Direction {
    Asc,
    Desc,
}

impl Direction {
    const fn try_from_int(value: i32) -> Option<Direction> {
        Some(match value.signum() {
            1 => Direction::Asc,
            -1 => Direction::Desc,
            _ => return None,
        })
    }
}

#[derive(Error, Debug, Copy, Clone, PartialEq)]
enum U32OperationError {
    #[error("Failed to cast as int: {0:?}")]
    TryFromIntError(#[from] TryFromIntError),
    #[error("Distance overflow")]
    DistanceOverflow,
}

#[derive(Error, Debug, Copy, Clone, PartialEq)]
enum IsNotSafeError {
    #[error("U32 Operation Error ix: {index}, error: {error:?}")]
    U32OperationError { index: usize, error: U32OperationError },
    #[error("Invalid Distance {distance}, ix: {index}")]
    InvalidDistance { index: usize, distance: u32 },
    #[error("Mixed Direction, expected {expected_direction:?}, ix: {index}")]
    MixedDirection { index: usize, expected_direction: Direction },
}

impl IsNotSafeError {
    fn index(&self) -> usize {
        match self {
            IsNotSafeError::U32OperationError { index, .. } => *index,
            IsNotSafeError::InvalidDistance { index, .. } => *index,
            IsNotSafeError::MixedDirection { index, .. } => *index,
        }
    }
    fn from_u32_op_error(index: usize, error: impl Into<U32OperationError>) -> Self {
        Self::U32OperationError { index, error: error.into() }
    }
}

fn validate_report_item(last_direction: &mut Option<Direction>, index: usize, distance: i32) -> Result<(), IsNotSafeError> {
    const ASCENDING: Option<Direction> = Some(Direction::Asc);
    const DESCENDING: Option<Direction> = Some(Direction::Desc);
    const ASCENDING_TO_DESCENDING: (Option<Direction>, Option<Direction>) = (ASCENDING, DESCENDING);
    const DESCENDING_TO_ASCENDING: (Option<Direction>, Option<Direction>) = (DESCENDING, ASCENDING);
    const ASCENDING_CONT: (Option<Direction>, Option<Direction>) = (ASCENDING, ASCENDING);
    const DESCENDING_CONT: (Option<Direction>, Option<Direction>) = (DESCENDING, DESCENDING);

    let next_direction = Direction::try_from_int(distance);

    let distance_abs = u32::try_from(distance.abs()).map_err(|err| IsNotSafeError::from_u32_op_error(index, err))?;

    match (*last_direction, next_direction) {
        // commit to the first distance we've determined
        (None, next_direction @ Some(_)) => {
            *last_direction = next_direction;
        },
        // distance mismatch
        ASCENDING_TO_DESCENDING | DESCENDING_TO_ASCENDING => {
            let Some(&last_direction) = last_direction.as_ref() else { unreachable!() };
            return Err(IsNotSafeError::MixedDirection { index, expected_direction: last_direction });
        },
        // Any two adjacent levels differ by at least one.
        (_, None) => {
            return Err(IsNotSafeError::InvalidDistance { index, distance: distance_abs });
        }
        ASCENDING_CONT | DESCENDING_CONT => {},
    }

    // Any two adjacent levels differ at most three.
    if distance_abs > 3 {
        return Err(IsNotSafeError::InvalidDistance { index, distance: distance_abs });
    }
    Ok(())
}

fn u32_distance(a: u32, b: u32) -> Result<i32, U32OperationError> {
    let value = match i32::try_from(a) {
        Ok(value) => value,
        Err(err) => {
            return Err(err.into());
        }
    };
    let next = match i32::try_from(b) {
        Ok(value) => value,
        Err(err) => {
            return Err(err.into());
        }
    };
    let Some(distance) = value.checked_sub(next) else {
        return Err(U32OperationError::DistanceOverflow);
    };
    Ok(distance)
}

fn validation_details(data: &Vec<u32>) -> (Option<Direction>, Vec<IsNotSafeError>) {
    let mut errors = Vec::new();
    let mut direction = None;
    let mut peekable = data.into_iter().enumerate().peekable();
    while let Some((ix, &value)) = peekable.next() {
        if let Some((next_ix, &next)) = peekable.peek().cloned() {
            let distance = match u32_distance(next, value) {
                Ok(distance) => distance,
                Err(err) => {
                    errors.push(IsNotSafeError::U32OperationError { index: ix, error: err });
                    continue;
                }
            };
            validate_report_item(&mut direction, next_ix, distance).unwrap_or_else(|err| errors.push(err))
        }
        else {
            // single item set, or no more items
            break;
        };
    }
    (direction, errors)
}

fn is_single_or_adjacent(items: &[usize]) -> bool {
    if items.len() == 1 {
        true
    } else if items.len() == 2 {
        items[0] == items[1] - 1
    } else {
        false
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = advent_of_code::read_input_lines(2);
    let mut reports = Vec::new();
    for input_line in lines {
        let input_line = input_line?;
        let data = input_line
            .trim()
            .split_whitespace()
            .map(|x| x.parse::<u32>().expect("Invalid input"))
            .collect::<Vec<_>>();
        reports.push(Report::from_data(data));
    }

    fn is_report_safe(report: &Report) -> bool {
        report.fix_cloned().map(|(_, report)| report).is_some()
    }

    let answer = reports.into_iter().filter(is_report_safe).count();
    println!("Answer: {answer}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_validation_invalid_distance() {
        let report = Report::from_data(vec![1, 2, 9]);
        assert_eq!(
            report.indexed_errors,
            vec![
                IsNotSafeError::InvalidDistance { index: 2, distance: 7 },
            ],
        );
    }
    #[test]
    fn test_validation_mixed_direction() {
        let report = Report::from_data(vec![2, 5, 4, 3]);
        assert_eq!(
            report.indexed_errors,
            vec![
                IsNotSafeError::MixedDirection { index: 2, expected_direction: Direction::Asc },
                IsNotSafeError::MixedDirection { index: 3, expected_direction: Direction::Asc },
            ],
        );
    }
    #[test]
    fn test_mixed_direction_removal() {
        let report = Report::from_data(vec![9, 7, 8]);
        let report = report.remove_bad_item_cloned(1);
        assert!(report.is_safe());

        let report = Report::from_data(vec![9, 7, 8]);
        let report = report.remove_bad_item_cloned(2);
        assert!(report.is_safe());

        let report = Report::from_data(vec![1, 2, 4, 3]);
        let report = report.remove_bad_item_cloned(2);
        assert!(report.is_safe());

        let report = Report::from_data(vec![1, 2, 4, 3]);
        let report = report.remove_bad_item_cloned(3);
        assert!(report.is_safe());
    }
    #[test]
    fn test_invalid_distance_removal() {
        let report = Report::from_data(vec![1, 2, 9]);
        let report = report.remove_bad_item_cloned(2);
        assert!(report.is_safe());
    }
    #[test]
    fn test_mixed_validation() {
        let report = Report::from_data(vec![1, 7, 6]);
        assert_eq!(
            report.indexed_errors,
            vec![
                IsNotSafeError::InvalidDistance { index: 1, distance: 6 },
                IsNotSafeError::MixedDirection { index: 2, expected_direction: Direction::Asc },
            ],
        );
    }
    #[test]
    fn test_mixed_removal() {
        let report = Report::from_data(vec![1, 7, 6]);
        let report = report.remove_bad_item_cloned(0);
        assert!(report.is_safe());
    }

    #[test]
    fn test_auto_mixed_removal() {
        let report = Report::from_data(vec![1, 7, 6]).fix();
        assert!(report.is_safe());

        let report = Report::from_data(vec![1, 7, 6, 8]).fix();
        assert!(!report.is_safe());
    }
    #[test]
    fn test() {
        fn fix_report(data: Vec<u32>) -> Vec<u32> {
            Report::from_data(data).fix().data
        }
        assert_eq!(fix_report(vec![5, 6, 7, 10, 13, 16, 13]), vec![5, 6, 7, 10, 13, 16]);
        assert_eq!(fix_report(vec![19, 21, 24, 27, 28, 28]), vec![19, 21, 24, 27, 28]);
        assert_eq!(fix_report(vec![16, 18, 20, 21, 23, 25, 29]), vec![16, 18, 20, 21, 23, 25]);
        assert_eq!(fix_report(vec![44, 46, 48, 49, 52, 55, 56, 62]), vec![44, 46, 48, 49, 52, 55, 56]);
        assert_eq!(fix_report(vec![31, 32, 32, 33, 36]), vec![31, 32, 33, 36]);
        assert_eq!(fix_report(vec![67, 66, 68, 70, 71, 74, 76]), vec![66, 68, 70, 71, 74, 76]);
        assert_eq!(fix_report(vec![9, 6, 12, 13, 15]), vec![9, 12, 13, 15]);
        assert_eq!(fix_report(vec![80, 80, 81, 84, 86]), vec![80, 81, 84, 86]);
        assert_eq!(fix_report(vec![20, 24, 27, 28, 30, 33, 35]), vec![24, 27, 28, 30, 33, 35]);
        assert_eq!(fix_report(vec![39, 43, 40, 43, 44, 46]), vec![39, 40, 43, 44, 46]);
        assert_eq!(fix_report(vec![9, 14, 17, 18, 19, 21, 22, 23]), vec![14, 17, 18, 19, 21, 22, 23]);
        assert_eq!(fix_report(vec![34, 32, 29, 27, 24, 22, 20, 23]), vec![34, 32, 29, 27, 24, 22, 20]);
        assert_eq!(fix_report(vec![99, 97, 95, 92, 90, 89, 86, 86]), vec![99, 97, 95, 92, 90, 89, 86]);
        assert_eq!(fix_report(vec![23, 22, 21, 20, 17, 16, 13, 9]), vec![23, 22, 21, 20, 17, 16, 13]);
        assert_eq!(fix_report(vec![77, 74, 73, 71, 68, 65, 64, 59]), vec![77, 74, 73, 71, 68, 65, 64]);
        assert_eq!(fix_report(vec![31, 29, 27, 24, 25, 23]), vec![31, 29, 27, 24, 23]); // or [31, 29, 27, 25, 23]
        assert_eq!(fix_report(vec![12, 11, 8, 8, 5, 2, 1]), vec![12, 11, 8, 5, 2, 1]);
        assert_eq!(fix_report(vec![40, 41, 38, 37, 35, 33, 32]), vec![41, 38, 37, 35, 33, 32]);
        assert_eq!(fix_report(vec![28, 31, 26, 23, 21, 18]), vec![28, 26, 23, 21, 18]);
        assert_eq!(fix_report(vec![42, 42, 41, 39, 37, 35, 34, 31]), vec![42, 41, 39, 37, 35, 34, 31]);
        assert_eq!(fix_report(vec![20, 16, 13, 12, 9, 8]), vec![16, 13, 12, 9, 8]);
        assert_eq!(fix_report(vec![70, 65, 63, 62, 61, 58, 56, 53]), vec![65, 63, 62, 61, 58, 56, 53]);
        assert_eq!(fix_report(vec![52, 55, 57, 60, 63, 66, 63]), vec![52, 55, 57, 60, 63, 66]);
        assert_eq!(fix_report(vec![10, 11, 12, 13, 14, 16, 16]), vec![10, 11, 12, 13, 14, 16]);
        assert_eq!(fix_report(vec![84, 87, 88, 90, 91, 95]), vec![84, 87, 88, 90, 91]);
        assert_eq!(fix_report(vec![12, 14, 17, 19, 20, 21, 26]), vec![12, 14, 17, 19, 20, 21]);
        assert_eq!(fix_report(vec![46, 49, 48, 51, 54]), vec![46, 49, 51, 54]); // or [46, 48, 51, 54]
        assert_eq!(fix_report(vec![13, 14, 17, 17, 20]), vec![13, 14, 17, 20]);
        assert_eq!(fix_report(vec![34, 33, 34, 35, 38, 41, 42, 45]), vec![33, 34, 35, 38, 41, 42, 45]);
        assert_eq!(fix_report(vec![31, 31, 33, 36, 38, 40, 42, 45]), vec![31, 33, 36, 38, 40, 42, 45]);
        assert_eq!(fix_report(vec![6, 10, 11, 13, 16]), vec![10, 11, 13, 16]);
        assert_eq!(fix_report(vec![72, 77, 79, 80, 81, 83]), vec![77, 79, 80, 81, 83]);
        assert_eq!(fix_report(vec![35, 40, 37, 38, 41]), vec![35, 37, 38, 41]);
        assert_eq!(fix_report(vec![16, 13, 12, 9, 8, 11]), vec![16, 13, 12, 9, 8]);
        assert_eq!(fix_report(vec![68, 67, 66, 65, 62, 62]), vec![68, 67, 66, 65, 62]);
        assert_eq!(fix_report(vec![46, 45, 44, 42, 40, 36]), vec![46, 45, 44, 42, 40]);
        assert_eq!(fix_report(vec![49, 48, 47, 46, 43, 42, 41, 34]), vec![49, 48, 47, 46, 43, 42, 41]);
        assert_eq!(fix_report(vec![93, 92, 91, 88, 90, 88]), vec![93, 92, 91, 90, 88]);
        assert_eq!(fix_report(vec![21, 20, 20, 19, 16, 14, 13]), vec![21, 20, 19, 16, 14, 13]);
        assert_eq!(fix_report(vec![19, 17, 16, 14, 10, 13]), vec![19, 17, 16, 14, 13]);
        assert_eq!(fix_report(vec![63, 66, 63, 60, 57, 55]), vec![66, 63, 60, 57, 55]);
        assert_eq!(fix_report(vec![81, 82, 78, 77, 75, 72, 69, 68]), vec![81, 78, 77, 75, 72, 69, 68]);
        assert_eq!(fix_report(vec![65, 68, 62, 60, 57]), vec![65, 62, 60, 57]);
        assert_eq!(fix_report(vec![42, 42, 41, 39, 36, 35, 34, 31]), vec![42, 41, 39, 36, 35, 34, 31]);
        assert_eq!(fix_report(vec![43, 39, 37, 36, 34, 33, 32, 29]), vec![39, 37, 36, 34, 33, 32, 29]);
        assert_eq!(fix_report(vec![61, 54, 52, 49, 46, 45, 43]), vec![54, 52, 49, 46, 45, 43]);
        assert_eq!(fix_report(vec![48, 43, 45, 43, 42]), vec![48, 45, 43, 42]);
        assert_eq!(fix_report(vec![76, 73, 72, 70, 70, 68, 66, 63]), vec![76, 73, 72, 70, 68, 66, 63]);
        assert_eq!(fix_report(vec![47, 47, 45, 44, 41, 39, 38, 37]), vec![47, 45, 44, 41, 39, 38, 37]);
        assert_eq!(fix_report(vec![36, 38, 40, 43, 46, 51]), vec![36, 38, 40, 43, 46]);
        assert_eq!(fix_report(vec![67, 66, 67, 70, 72, 74, 77, 80]), vec![66, 67, 70, 72, 74, 77, 80]);
        assert_eq!(fix_report(vec![67, 70, 71, 72, 73, 75, 78, 78]), vec![67, 70, 71, 72, 73, 75, 78]);
        assert_eq!(fix_report(vec![87, 84, 83, 81, 78, 77, 76, 76]), vec![87, 84, 83, 81, 78, 77, 76]);
        assert_eq!(fix_report(vec![55, 53, 57, 60, 63, 64]), vec![55, 57, 60, 63, 64]);
        assert_eq!(fix_report(vec![47, 49, 50, 52, 53, 56, 59, 63]), vec![47, 49, 50, 52, 53, 56, 59]);
        assert_eq!(fix_report(vec![62, 59, 57, 55, 54, 51, 50, 44]), vec![62, 59, 57, 55, 54, 51, 50]);
        assert_eq!(fix_report(vec![12, 16, 15, 17, 19]), vec![12, 15, 17, 19]);
        assert_eq!(fix_report(vec![48, 43, 42, 41, 40, 37]), vec![43, 42, 41, 40, 37]);
        assert_eq!(fix_report(vec![23, 20, 17, 15, 13, 11, 10, 6]), vec![23, 20, 17, 15, 13, 11, 10]);
        assert_eq!(fix_report(vec![57, 60, 63, 66, 67, 69, 71, 68]), vec![57, 60, 63, 66, 67, 69, 71]);
        assert_eq!(fix_report(vec![49, 52, 49, 46, 43, 41]), vec![52, 49, 46, 43, 41]);
        assert_eq!(fix_report(vec![22, 27, 28, 31, 32, 33]), vec![27, 28, 31, 32, 33]);
        assert_eq!(fix_report(vec![51, 55, 58, 59, 60, 63, 66]), vec![55, 58, 59, 60, 63, 66]);
        assert_eq!(fix_report(vec![56, 55, 52, 51, 50, 48, 44, 45]), vec![56, 55, 52, 51, 50, 48, 45]);
        assert_eq!(fix_report(vec![68, 64, 61, 58, 55, 52, 50]), vec![64, 61, 58, 55, 52, 50]);
    }
}
