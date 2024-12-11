use std::collections::VecDeque;
use std::error::Error;
use std::fmt::Debug;
use std::num::ParseIntError;
use std::str::FromStr;
use thiserror::Error;
use advent_of_code::read_input_lines;

fn main() -> Result<(), Box<dyn Error>> {
    let lines = read_input_lines(7).map(|line| line.expect("failed to read input"));
    let mut total = 0;
    for line in lines {
        let parse_line_result = parse_line(&line)?;
        if let Some(calibrated_total) = calibrate_equation(parse_line_result) {
            total += calibrated_total;
        }
    }
    println!("Answer: {}", total);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum OperatorIdentity {
    Add,
    Multiply,
    Concat,
}

impl OperatorIdentity {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "+",
            Self::Multiply => "*",
            Self::Concat => "||",
        }
    }
    fn operator(&self, lhs: i64, rhs: i64) -> Operator {
        match self {
            Self::Add => Operator::Add(lhs, rhs),
            Self::Multiply => Operator::Multiply(lhs, rhs),
            Self::Concat => Operator::Concat(lhs, rhs),
        }
    }
    fn eval(&self, lhs: i64, rhs: i64) -> i64 {
        self.operator(lhs, rhs).eval()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Operator {
    Add(i64, i64),
    Multiply(i64, i64),
    Concat(i64, i64),
}

impl Operator {
    fn eval(&self) -> i64 {
        match self {
            Operator::Add(a, b) => a + b,
            Operator::Multiply(a, b) => a * b,
            Operator::Concat(a, b) => format!("{a}{b}").parse().unwrap_or_else(|err| panic!("{err:?}")),
        }
    }
    fn identity(&self) -> OperatorIdentity {
        match self {
            Operator::Add(_, _) => OperatorIdentity::Add,
            Operator::Multiply(_, _) => OperatorIdentity::Multiply,
            Operator::Concat(_, _) => OperatorIdentity::Concat,
        }
    }
}

fn calibrate_equation(parse_line_result: ParseLineResult) -> Option<i64> {
    const OPS: [OperatorIdentity; 3] = [OperatorIdentity::Add, OperatorIdentity::Multiply, OperatorIdentity::Concat];
    let total_ops = OPS.len();
    let operator_slots = parse_line_result.numbers.len().saturating_sub(1);
    // single number
    if operator_slots == 0 {
        if parse_line_result.total == parse_line_result.numbers.iter().sum() {
            return Some(parse_line_result.total);
        }
        return None;
    }
    let total_combinations = total_ops.pow(operator_slots as u32);
    for n in 0..total_combinations {
        let mut n = n;
        let mut numbers = parse_line_result.numbers.clone().into_iter().collect::<VecDeque<i64>>();
        let Some(mut lhs) = numbers.pop_front() else { unreachable!() };
        // let mut debug_equation = vec![lhs.to_string()];
        for _ in 0..operator_slots {
            let Some(rhs) = numbers.pop_front() else { unreachable!() };
            let operator = OPS[n % total_ops];
            // debug_equation.push(operator.as_str().to_owned());
            // debug_equation.push(rhs.to_string());
            lhs = operator.eval(lhs, rhs);
            n /= total_ops;
        }
        let is_match = lhs == parse_line_result.total;
        if is_match {
            // debug_equation.push("=".to_string());
            // debug_equation.push(lhs.to_string());
            // println!("{}", debug_equation.into_iter().collect::<String>());
            return Some(lhs);
        }
    }
    None
}

#[derive(Error, Debug, Clone, PartialEq)]
#[error("ParseVecError: {ix}: {err:?}")]
struct ParseVecError<T>
where
    T: FromStr + Debug + Clone,
    T::Err: Debug,
{
    ix: usize,
    err: T::Err,
}

#[derive(Error, Debug, Clone, PartialEq)]
enum ParseTotalError {
    #[error("ParseTotalError: {0:?}")]
    ParseIntError(String, ParseIntError),
}

#[derive(Error, Debug, Clone, PartialEq)]
enum ParseNumbersError {
    #[error("ParseNumbersError: {0:?}")]
    ParseIntError(String, ParseVecError<i64>),
    #[error("ParseNumbersError: invalid format")]
    InvalidFormat(String),
}

#[derive(Error, Debug, Clone, PartialEq)]
enum ParseError {
    #[error("ParseError::ParseTotalError: {0:?}")]
    ParseTotalError(#[from] ParseTotalError),
    #[error("ParseError::ParseNumbersError: {0:?}")]
    ParseNumbersError(#[from] ParseNumbersError),
    #[error("ParseError: invalid format")]
    InvalidFormat(String),
}

#[derive(Debug, Clone, PartialEq)]
struct ParseLineResult {
    total: i64,
    numbers: Vec<i64>,
}

fn parse_line(line: &str) -> Result<ParseLineResult, ParseError> {
    let line = line.trim();
    if line.is_empty() {
        return Err(ParseError::InvalidFormat(line.to_owned()))?;
    }
    let mut parts = line.split(':');

    let total = {
        let Some(total) = parts.next() else {
            unreachable!()
        };
        total.parse()
            .map_err(|err| ParseTotalError::ParseIntError(line.to_owned(), err))?
    };

    let numbers = {
        let Some(numbers) = parts.next() else {
            return Err(ParseNumbersError::InvalidFormat(line.to_owned()))?;
        };
        let numbers = numbers.trim().split_whitespace().collect::<Vec<_>>();
        parse_vec(numbers)
            .map_err(|err| ParseNumbersError::ParseIntError(line.to_owned(), err))?
    };

    if parts.next().is_some() {
        return Err(ParseError::InvalidFormat(line.to_owned()));
    };

    Ok(ParseLineResult { total, numbers })
}

fn parse_vec<T>(items: Vec<&str>) -> Result<Vec<T>, ParseVecError<T>>
where
    T: FromStr + Debug + Clone,
    T::Err: Debug,
{
    let mut results = Vec::with_capacity(items.len());
    for (ix, item) in items.into_iter().enumerate() {
        let result = item.parse::<T>().map_err(|err| ParseVecError { ix, err })?;
        results.push(result);
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use advent_of_code::utils::string::deformat_string;
    use super::*;

    #[test]
    fn parse_line_test() {
        let line = "1: 2 3 4 5 6";
        let expected_parse_line_result = ParseLineResult {
            total: 1,
            numbers: vec![2, 3, 4, 5, 6],
        };
        let result = parse_line(line);
        assert_eq!(result, Ok(expected_parse_line_result));
    }

    #[test]
    fn parse_line_empty_test() {
        let line = "";
        let result = parse_line(line);
        assert_eq!(result, Err(ParseError::InvalidFormat(line.to_owned())));
    }

    #[test]
    fn exmaples() -> Result<(), Box<dyn std::error::Error>> {
        let input = deformat_string("
            190: 10 19
            3267: 81 40 27
            83: 17 5
            156: 15 6
            7290: 6 8 6 15
            161011: 16 10 13
            192: 17 8 14
            21037: 9 7 18 13
            292: 11 6 16 20
        ");
        let lines = input.lines();
        let mut total = 0;
        for line in lines {
            let parse_line_result = parse_line(&line)?;
            if let Some(calibrated_total) = calibrate_equation(parse_line_result) {
                total += calibrated_total;
            }
        }
        assert_eq!(total, 11387);
        Ok(())
    }
}
