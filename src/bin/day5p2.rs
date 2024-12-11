use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use thiserror::Error;
use advent_of_code::read_input_lines;

fn main() {
    let lines = read_input_lines(5).map(|line| line.expect("failed to read input"));
    let (rules, updates) = parse_input(lines);

    let mut sum = 0;
    for mut update in updates {
        if let Err(err) = validate_update(&rules, &update) {
            if update.len() == 0 || update.len() % 2 == 0 {
                panic!("expected odd length update, got: {update:?}");
            }

            fix_update(&rules, &mut update);

            let mid_ix = update.len() / 2;
            let Some(mid_page) = update.get(mid_ix) else {
                unreachable!()
            };
            sum += mid_page;
        }
    }
    let answer = sum;
    println!("Answer: {answer}");
}

type Rules = HashMap<u32, HashSet<u32>>;
type Update = Vec<u32>;
type Updates = Vec<Update>;

fn parse_u32(s: &str) -> u32 {
    s.parse::<u32>().unwrap_or_else(|err| panic!("input: {s}, error: {err:?}"))
}

fn parse_update(input: &str) -> Update {
    input.split(',').map(parse_u32).collect::<Vec<_>>()
}

fn parse_updates(updates: &mut Updates, input: &str) {
    for line in input.lines() {
        if line.trim().is_empty() {
            break;
        }
        let page_sequence = parse_update(line);
        updates.push(page_sequence);
    }
}

fn parse_rule(input: &str) -> (u32, u32) {
    let mut parts = input.split('|');
    let item = parts.next().map(parse_u32).unwrap_or_else(|| panic!("unexpected format: {input}"));
    let after = parts.next().map(parse_u32).unwrap_or_else(|| panic!("unexpected format: {input}"));
    assert_eq!(parts.next(), None);
    (item, after)
}

fn parse_rules(rules: &mut Rules, input: &str) {
    for line in input.lines() {
        if line.trim().is_empty() {
            break;
        }
        let (item , after) = parse_rule(line);
        let entry = rules.entry(item).or_default();
        entry.insert(after);
    }
}

#[cfg(test)]
fn parse_input_string(input: &str) -> (Rules, Vec<Vec<u32>>) {
    parse_input(input.lines())
}

fn parse_input<I, S>(mut lines: I) -> (Rules, Vec<Vec<u32>>)
where
    I: Iterator<Item = S>,
    S: AsRef<str>
{
    let mut rules = Rules::new();
    let mut updates = Updates::default();
    while let Some(line) = lines.next() {
        let line = line.as_ref();
        if line.trim().is_empty() {
            break;
        }
        parse_rules(&mut rules, line);
    }
    while let Some(line) = lines.next() {
        let line = line.as_ref();
        if line.trim().is_empty() {
            break;
        }
        parse_updates(&mut updates, line);
    }
    (rules, updates)
}

#[derive(Error, Debug, Clone, Copy, Default, PartialEq)]
struct InvalidUpdate {
    page: u32,
    page_ix: usize,
    preceding_violation_page: u32,
    preceding_violation_page_ix: usize,
}

impl Display for InvalidUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "InvalidUpdate {{ page: {}, preceding violation: {} }}", self.page, self.preceding_violation_page)
    }
}

fn validate_update(rules: &Rules, update: &Update) -> Result<(), InvalidUpdate> {
    for (page_ix, &page) in update.iter().enumerate().rev() {
        if let Some(after) = rules.get(&page) {
            for violating_page in after {
                // search all numbers before to see if they violate the after rule
                if let Some(item_ix) = update[0..page_ix].iter().rposition(|item| item == violating_page) {
                    return Err(InvalidUpdate {
                        page,
                        page_ix,
                        preceding_violation_page: *violating_page,
                        preceding_violation_page_ix: item_ix,
                    });
                }
            }
        }
    }
    Ok(())
}

fn fix_update(rules: &Rules, update: &mut Update) {
    let mut max_iterations: usize = 100;
    let original_update = update.clone();
    while let Err(err) = validate_update(rules, update) {
        let page = update.remove(err.preceding_violation_page_ix);
        update.insert(err.page_ix, page);
        max_iterations = max_iterations.saturating_sub(1);
        if max_iterations == 0 {
            println!("original_update: {original_update:?}");
            println!();
            println!("rules:");
            for page in update.iter() {
                println!("{page}: {:?}", rules.get(&page).map(|rule| {
                    let mut filtered_after = rule.iter().filter(|&page| update.contains(page)).collect::<Vec<_>>();
                    filtered_after.sort();
                    filtered_after
                }));
            }
            println!();
            println!("last validation error: {err:?}");
            panic!("max iterations reached! {update:?}")
        }
    }
}

#[cfg(test)]
mod tests {
    use advent_of_code::utils::string::deformat_string;
    use super::*;

    #[test]
    fn parse_rule_test() {
        let (item, after) = parse_rule("47|53");
        assert_eq!(item, 47);
        assert_eq!(after, 53);
    }

    #[test]
    fn parse_update_test() {
        let update = parse_update("47,53,61,29");
        assert_eq!(update, vec![47, 53, 61, 29]);
    }

    #[test]
    fn parse_rules_test() {
        let mut rules = Rules::new();
        parse_rules(&mut rules, &deformat_string("
            47|53
            97|13
            97|61
            97|47
            75|29
            61|13
        "));

        let mut expected_rules = Rules::new();
        expected_rules.entry(47).or_insert_with(HashSet::new).insert(53);
        expected_rules.entry(97).or_insert_with(HashSet::new).insert(13);
        expected_rules.entry(97).or_insert_with(HashSet::new).insert(61);
        expected_rules.entry(97).or_insert_with(HashSet::new).insert(47);
        expected_rules.entry(75).or_insert_with(HashSet::new).insert(29);
        expected_rules.entry(61).or_insert_with(HashSet::new).insert(13);

        assert_eq!(rules, expected_rules);
    }

    #[test]
    fn parse_updates_test() {
        let mut updates = Updates::default();
        parse_updates(&mut updates, &deformat_string("
            47,53,61,29
            97,61,53,29,13
            75,29,13
            75,97,47,61,53
            61,13,29
            97,13,75,29,47
        "));
        let expected_updates = vec![
            vec![47, 53, 61, 29],
            vec![97, 61, 53, 29, 13],
            vec![75, 29, 13],
            vec![75, 97, 47, 61, 53],
            vec![61, 13, 29],
            vec![97, 13, 75, 29, 47],
        ];
        assert_eq!(updates, expected_updates);
    }

    #[test]
    fn parse_input_string_test() {
        let input = deformat_string("
                1|2
                2|3
                2|5

                1,2,3
                1,2,5
            ");

        let (rules, updates) = parse_input_string(&input);
        assert_eq!(rules, Rules::from([
            (1, HashSet::from([2])),
            (2, HashSet::from([3, 5])),
        ]));
        assert_eq!(updates, Updates::from([
            vec![1, 2, 3],
            vec![1, 2, 5],
        ]));
    }

    #[test]
    fn validate_update_test() {
        let rules = Rules::from([
            (1, HashSet::from([2])),
            (2, HashSet::from([3, 5])),
        ]);
        assert_eq!(validate_update(&rules, &vec![1, 2, 3]), Ok(()));
        assert_eq!(validate_update(&rules, &vec![1, 2, 5]), Ok(()));
        assert_eq!(validate_update(&rules, &vec![2, 1, 3]), Err(InvalidUpdate {
            page: 1,
            page_ix: 1,
            preceding_violation_page: 2,
            preceding_violation_page_ix: 0,
        }))
    }

    #[test]
    fn fix_update_test() {
        let mut update = vec![1, 5, 3, 2];
        fix_update(&Rules::from([
            (1, HashSet::from([2])),
            (2, HashSet::from([3])),
        ]), &mut update);
        assert_eq!(update, vec![1, 5, 2, 3]);
    }

    #[test]
    fn example() {
        let input = deformat_string("
            47|53
            97|13
            97|61
            97|47
            75|29
            61|13
            75|53
            29|13
            97|29
            53|29
            61|53
            97|53
            61|29
            47|13
            75|47
            97|75
            47|61
            75|61
            47|29
            75|13
            53|13

            75,47,61,53,29
            97,61,53,29,13
            75,29,13
            75,97,47,61,53
            61,13,29
            97,13,75,29,47
        ");

        let (rules, updates) = parse_input_string(&input);

        let mut sum = 0;
        for mut update in updates {
            if let Err(err) = validate_update(&rules, &update) {
                if update.len() == 0 || update.len() % 2 == 0 {
                    panic!("expected odd length update, got: {update:?}");
                }

                fix_update(&rules, &mut update);

                let mid_ix = update.len() / 2;
                let Some(mid_page) = update.get(mid_ix) else {
                    unreachable!()
                };
                sum += mid_page;
            }
        }
        assert_eq!(sum, 123);
    }
}
