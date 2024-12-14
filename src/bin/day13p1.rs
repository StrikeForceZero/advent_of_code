use anyhow::anyhow;
use glam::UVec2;
use itertools::iproduct;
use lazy_static::lazy_static;
use regex::Regex;
use advent_of_code::read_input_lines;

fn main() -> anyhow::Result<()> {
    let lines = read_input_lines(13).map(|line| line.expect("failed to read line")).collect::<Vec<_>>();
    // this is dirty but meh
    let mut lines = lines.iter().map(|line| line.as_str());
    let mut tokens = 0;
    while let Ok(section) = parse_section(&mut lines) {
        if let Ok(instance) = section.solve() {
            println!("solved section with token cost: {}", instance.token_cost());
            tokens += instance.token_cost();
        } else {
            println!("failed to solve section");
        }
    }
    println!("answer: {tokens}");
    Ok(())
}

lazy_static! {
    static ref BUTTON_LINE: Regex = Regex::new("Button [AB]: X\\+(\\d+), Y\\+(\\d+)").expect("invalid regex");
    static ref PRIZE_LINE: Regex = Regex::new("Prize: X=(\\d+), Y=(\\d+)").expect("invalid regex");
}

fn parse_button_line(line: &str) -> anyhow::Result<UVec2> {
    let Some(captures) = BUTTON_LINE.captures(line) else { return Err(anyhow::anyhow!("invalid button line: {line}")) };
    let x = captures.get(1).unwrap().as_str().parse::<u32>()?;
    let y = captures.get(2).unwrap().as_str().parse::<u32>()?;
    Ok(UVec2::new(x, y))
}

fn parse_prize_line(line: &str) -> anyhow::Result<UVec2> {
    let Some(captures) = PRIZE_LINE.captures(line) else { return Err(anyhow::anyhow!("invalid prize line: {line}")) };
    let x = captures.get(1).unwrap().as_str().parse::<u32>()?;
    let y = captures.get(2).unwrap().as_str().parse::<u32>()?;
    Ok(UVec2::new(x, y))
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
struct Section {
    button_a: UVec2,
    button_b: UVec2,
    prize: UVec2,
}

impl Section {
    fn solve(self) -> anyhow::Result<Instance> {
        let coefficients = 1..=100;

        let mut best_instance: Option<Instance> = None;
        // Generate all combinations of coefficients for `a` and `b`
        for (coeff_a, coeff_b) in iproduct!(coefficients.clone(), coefficients.clone()) {
            let mut instance = Instance::new(self);
            instance.a_times = coeff_a;
            instance.b_times = coeff_b;
            if instance.is_at_spot() && !instance.too_many_a_presses() && !instance.too_many_b_presses() {
                if let Some(best) = best_instance {
                    if instance.token_cost() < best.token_cost() {
                        best_instance = Some(instance);
                    }
                } else {
                    best_instance = Some(instance);
                }
            }
        }
        if let Some(best) = best_instance {
            Ok(best)
        } else {
            Err(anyhow!("failed to solve"))
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
struct Instance {
    section: Section,
    a_times: u32,
    b_times: u32,
}
impl Instance {
    fn new(section: Section) -> Self {
        Self {
            section,
            ..Default::default()
        }
    }
    fn is_done(&self) -> bool {
        self.is_at_spot() || self.too_many_a_presses() || self.too_many_b_presses()
    }
    fn is_at_spot(&self) -> bool {
        self.pos() == self.section.prize
    }
    fn pos(&self) -> UVec2 {
        self.a_times * self.section.button_a + self.b_times * self.section.button_b
    }
    fn token_cost(&self) -> u32 {
        self.a_times * 1 + self.b_times * 3
    }
    fn too_many_a_presses(&self) -> bool {
        self.a_times >= 100
    }
    fn too_many_b_presses(&self) -> bool {
        self.b_times >= 100
    }
    fn a(&mut self) -> anyhow::Result<()> {
        if self.is_at_spot() {
            return Err(anyhow!("already at spot"));
        }
        if self.too_many_a_presses() {
            return Err(anyhow!("pressed a too many times"));
        }
        self.a_times += 1;
        Ok(())
    }
    fn b(&mut self) -> anyhow::Result<()> {
        if self.is_at_spot() {
            return Err(anyhow!("already at spot"));
        }
        if self.too_many_b_presses() {
            return Err(anyhow!("pressed b too many times"));
        }
        self.a_times += 1;
        Ok(())
    }
}


fn parse_section<'a>(lines: &mut impl Iterator<Item=&'a str>) -> anyhow::Result<Section> {
    let section = Section {
        button_a: parse_button_line(lines.next().ok_or(anyhow!("missing button A line"))?)?,
        button_b: parse_button_line(lines.next().ok_or(anyhow!("missing button B line"))?)?,
        prize: parse_prize_line(lines.next().ok_or(anyhow!("missing prize line"))?)?,
    };
    let _ = lines.next();
    Ok(section)
}

#[cfg(test)]
mod tests {
    use advent_of_code::utils::string::deformat_string;
    use super::*;

    #[test]
    fn examples() -> anyhow::Result<()> {
        let input = deformat_string("
            Button A: X+94, Y+34
            Button B: X+22, Y+67
            Prize: X=8400, Y=5400

            Button A: X+26, Y+66
            Button B: X+67, Y+21
            Prize: X=12748, Y=12176

            Button A: X+17, Y+86
            Button B: X+84, Y+37
            Prize: X=7870, Y=6450

            Button A: X+69, Y+23
            Button B: X+27, Y+71
            Prize: X=18641, Y=10279
        ");
        let mut lines = input.lines();
        let section = parse_section(&mut lines)?;
        assert_eq!(
            section.solve()?,
            Instance {
                section,
                a_times: 80,
                b_times: 40,
            }
        );
        let section = parse_section(&mut lines)?;
        assert!(section.solve().is_err());
        let section = parse_section(&mut lines)?;
        assert_eq!(
            section.solve()?,
            Instance {
                section,
                a_times: 38,
                b_times: 86,
            }
        );
        let section = parse_section(&mut lines)?;
        assert!(section.solve().is_err());
        Ok(())
    }
}
