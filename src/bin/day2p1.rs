use std::error::Error;
use std::num::TryFromIntError;
use thiserror::Error;

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

#[derive(Error, Debug)]
enum IsNotSafeError {
    #[error("U32 Operation Error: {0:?}")]
    U32OperationError(#[from] U32OperationError),
    #[error("Invalid Distance")]
    InvalidDistance,
    #[error("Mixed Direction")]
    MixedDirection,
}

#[derive(Error, Debug, Copy, Clone, PartialEq)]
enum U32OperationError {
    #[error("Failed to cast as int: {0:?}")]
    TryFromIntError(#[from] TryFromIntError),
    #[error("Distance overflow")]
    DistanceOverflow,
}

fn u32_distance(a: u32, b: u32) -> Result<i32, U32OperationError> {
    let a = i32::try_from(a)?;
    let b = i32::try_from(b)?;
    let Some(result) = a.checked_sub(b) else {
        return Err(U32OperationError::DistanceOverflow)
    };
    Ok(result)
}

fn validate_report(report: &Vec<u32>) -> Result<(), IsNotSafeError> {
    const ASCENDING: Option<Direction> = Some(Direction::Asc);
    const DESCENDING: Option<Direction> = Some(Direction::Desc);
    const ASCENDING_TO_DESCENDING: (Option<Direction>, Option<Direction>) = (ASCENDING, DESCENDING);
    const DESCENDING_TO_ASCENDING: (Option<Direction>, Option<Direction>) = (DESCENDING, ASCENDING);
    const ASCENDING_CONT: (Option<Direction>, Option<Direction>) = (ASCENDING, ASCENDING);
    const DESCENDING_CONT: (Option<Direction>, Option<Direction>) = (DESCENDING, DESCENDING);

    let mut last_direction = None;
    let mut peekable = report.iter().peekable();
    while let Some(&value) = peekable.next() {
        let Some(&next) = peekable.peek().cloned() else {
            break;
        };
        let distance = u32_distance(next, value)?;
        let next_direction = Direction::try_from_int(distance);

        match (last_direction, next_direction) {
            // commit to the first distance we've determined
            (None, next_direction @ Some(_)) => {
                last_direction = next_direction;
            },
            // distance mismatch
            ASCENDING_TO_DESCENDING | DESCENDING_TO_ASCENDING => {
                return Err(IsNotSafeError::MixedDirection)
            },
            // Any two adjacent levels differ by at least one.
            (_, None) => {
                return Err(IsNotSafeError::InvalidDistance);
            }
            ASCENDING_CONT | DESCENDING_CONT => {},
        }

        // Any two adjacent levels differ at most three.
        if distance.abs() > 3 {
            return Err(IsNotSafeError::InvalidDistance);
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let lines = advent_of_code::read_input_lines(2);
    let mut reports = Vec::new();
    for input_line in lines {
        let input_line = input_line?;
        let report = input_line
            .trim()
            .split_whitespace()
            .map(|x| x.parse::<u32>().expect("Invalid input"))
            .collect::<Vec<_>>();
        reports.push(report);
    }
    fn is_report_safe(report: &Vec<u32>) -> bool {
        if let Err(err) = validate_report(&report) {
            if let IsNotSafeError::U32OperationError(err) = err {
                panic!("U32 Operation Error: {err:?}");
            }
            return false
        }
        true
    }
    let answer = reports.into_iter().filter(is_report_safe).count();
    println!("Answer: {answer}");
    Ok(())
}
