use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter};
use itertools::Itertools;
use num::{Integer, ToPrimitive};
use advent_of_code::read_input;

fn main() -> anyhow::Result<()> {
    let input = read_input(11)?;
    let mut gen = StoneGenerator::try_from(input)?;
    for n in 0..75 {
        gen.blink();
        println!("{n}: {}", gen.stone_count());
    }
    let answer = gen.stone_count();
    println!("Answer: {answer}");
    Ok(())
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum EvenOddSplitResult {
    Zero,
    Odd(u64),
    Even((u64, u64)),
}

// TODO: this should probably be on a trait and impl for each infallible number types via macro
fn count_num_digits(num: u64) -> u64 {
    if num == 0 {
        1
    } else {
        let float = num.to_f64().filter(|float| float.is_finite()).unwrap_or_else(|| panic!("{num} could not be represented by a f64"));
        float.log10().floor() as u64 + 1
    }
}

fn split_even_length_number(num: u64) -> EvenOddSplitResult {
    if num == 0 {
        return EvenOddSplitResult::Zero;
    }

    // Count the number of digits
    let num_digits = count_num_digits(num);

    // Check if the number of digits is even
    if !num_digits.is_even() {
        return EvenOddSplitResult::Odd(num);
    }

    // Calculate the midpoint
    let mid: u64 = num_digits / 2;

    // Calculate the divisor to split the number
    // TODO: potentially infallible since u64::MAX passes
    let divisor = 10_u64.pow(u32::try_from(mid).unwrap_or_else(|err| panic!("count_num_digits({num}) / 2 = {mid} could not be represented by a u64: {err}")));
    let num = num as u64;

    // Split the number into two parts
    let left_part = num / divisor;
    // TODO: potentially infallible since u64::MAX passes
    let left_part = u64::try_from(left_part).unwrap_or_else(|err| panic!("{num} / {divisor} = {left_part} could not be represented by a u64: {err}"));

    let right_part = num % divisor;
    // TODO: potentially infallible since u64::MAX passes
    let right_part = u64::try_from(right_part).unwrap_or_else(|err| panic!("{num} % {divisor} = {right_part} could not be represented by a u64: {err}"));

    EvenOddSplitResult::Even((left_part, right_part))
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum StoneProcessResult {
    Single(Stone),
    Split((Stone, Stone)),
}

impl StoneProcessResult {
    fn for_each<F>(self, mut f: F) where F: FnMut(Stone) {
        match self {
            StoneProcessResult::Single(stone) => {
                f(stone);
            },
            StoneProcessResult::Split((a, b)) => {
                f(a);
                f(b);
            },
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Ord, PartialOrd, Eq, Hash)]
struct Stone(u64);

impl Stone {
    fn process(self) -> StoneProcessResult {
        let num = self.0;
        if num == 0 {
            StoneProcessResult::Single(Self(1))
        } else if let EvenOddSplitResult::Even((left, right)) = split_even_length_number(num) {
            StoneProcessResult::Split((Self(left), Self(right)))
        } else {
            StoneProcessResult::Single(Self(num * 2024))
        }
    }
}

impl Display for Stone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct StoneGenerator {
    stones: HashMap<Stone, usize>,
    blinks: usize,
    cache: HashMap::<Stone, StoneProcessResult>,
}

impl StoneGenerator {
    #[allow(dead_code)]
    /// blinks single time
    ///
    /// does not factor previous blinks
    fn blink(&mut self) -> &mut Self {
        self.next();
        self
    }
    #[allow(dead_code)]
    /// blinks given number of times.
    ///
    /// does not factor previous blinks
    fn blinks(&mut self, num: usize) -> &mut Self {
        for _ in 0..num {
            self.next();
        }
        self
    }
    #[allow(dead_code)]
    /// factors in past blinks and blinks remainder
    fn total_blinks(&mut self, num: usize) -> &mut Self {
        let blinks_left = num.saturating_sub(self.blinks);
        for _ in 0..blinks_left {
            self.next();
        }
        self
    }
    fn stone_count(&self) -> usize {
        self.stones.iter().map(|(_, count)| count).sum()
    }
}

impl Iterator for StoneGenerator {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        self.blinks += 1;
        let mut new_stones = HashMap::<Stone, usize>::new();
        for (&stone, count) in self.stones.iter_mut() {
            let cache_entry = self.cache.entry(stone).or_insert_with(|| stone.process());
            cache_entry.for_each(|stone| {
                let entry = new_stones.entry(stone).or_default();
                *entry += *count;
            });
            *count = 0;
        }
        for (new_stone, count) in new_stones {
            self.stones.insert(new_stone, count);
        }
        Some(self.stones.len())
    }
}

fn stones_from_str(s: &str) -> anyhow::Result<HashMap<Stone, usize>> {
    let raw_stones = s.trim().split_whitespace();
    let mut stones = HashMap::<Stone, usize>::new();
    for stone in raw_stones.into_iter() {
        let stone = stone.parse::<u64>()?;
        let stone = Stone(stone);
        *stones.entry(stone).or_default() += 1;
    }
    Ok(stones)
}

impl TryFrom<&str> for StoneGenerator {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let stones = stones_from_str(value)?;
        Ok(StoneGenerator {
            stones,
            ..Default::default()
        })
    }
}

impl TryFrom<String> for StoneGenerator {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&String> for StoneGenerator {
    type Error = anyhow::Error;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl Display for StoneGenerator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (ix, (stone, count)) in self.stones.iter().enumerate() {
            let whitespace = if ix == 0 { "" } else { " " };
            write!(f, "{whitespace}{stone}:{count}")?;
        }
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn examples() -> anyhow::Result<()> {
        let mut generator = StoneGenerator::try_from("125 17")?;
        assert_eq!(generator.blinks(1).to_string(), "253000 1 7");
        assert_eq!(generator.total_blinks(2).to_string(), "253 0 2024 14168");
        assert_eq!(generator.total_blinks(3).to_string(), "512072 1 20 24 28676032");
        assert_eq!(generator.total_blinks(4).to_string(), "512 72 2024 2 0 2 4 2867 6032");
        assert_eq!(generator.total_blinks(5).to_string(), "1036288 7 2 20 24 4048 1 4048 8096 28 67 60 32");
        assert_eq!(generator.total_blinks(6).to_string(), "2097446912 14168 4048 2 0 2 4 40 48 2024 40 48 80 96 2 8 6 7 6 0 3 2");
        assert_eq!(generator.total_blinks(25).stones.len(), 55312);
        Ok(())
    }

    #[test]
    fn count_num_digits_test() -> anyhow::Result<()> {
        assert_eq!(count_num_digits(0), 1);
        assert_eq!(count_num_digits(1), 1);
        assert_eq!(count_num_digits(10), 2);
        assert_eq!(count_num_digits(100), 3);
        assert_eq!(count_num_digits(1000), 4);
        assert_eq!(count_num_digits(u32::MAX as u64), 10);
        assert_eq!(count_num_digits(u64::MAX), 20);
        Ok(())
    }

    #[test]
    fn split_even_length_number_test() -> anyhow::Result<()> {
        assert_eq!(split_even_length_number(0), EvenOddSplitResult::Zero);
        assert_eq!(split_even_length_number(1), EvenOddSplitResult::Odd(1));
        assert_eq!(split_even_length_number(10), EvenOddSplitResult::Even((1, 0)));
        assert_eq!(split_even_length_number(100), EvenOddSplitResult::Odd(100));
        assert_eq!(split_even_length_number(1000), EvenOddSplitResult::Even((10, 0)));
        assert_eq!(split_even_length_number(u32::MAX as u64), EvenOddSplitResult::Even((42949, 67295)));
        assert_eq!(split_even_length_number(u64::MAX), EvenOddSplitResult::Even((1844674407, 3709551615)));
        Ok(())
    }
}
