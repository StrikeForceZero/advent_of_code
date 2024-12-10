use std::collections::HashMap;
use std::fmt::Write;
use itertools::{Itertools};
use thiserror::Error;
use advent_of_code::read_input;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = read_input(9);
    let line = input.next().expect("input empty")?;
    let disk_map = DeflatedDiskMap::try_from(line)?.inflate();
    assert!(input.next().is_none(), "input expected to be single line");
    // println!("Disk map: {disk_map}");
    let compact_disk_map = disk_map.compact();
    // println!("Compact Disk map: {compact_disk_map}");
    let checksum = compact_disk_map.checksum();
    println!("Checksum: {checksum}");
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
struct FileBlock {
    id: usize,
    span: usize,
}

#[derive(Debug, Clone, PartialEq)]
enum CompactBlock {
    File(FileBlock),
    FreeSpace(usize),
}

#[derive(Debug, Clone, PartialEq)]
enum CompactResult {
    DiskMap(DiskMap),
    CompactedDiskMap(CompactedDiskMap),
}

impl std::fmt::Display for CompactResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CompactResult::DiskMap(disk_map) => write!(f, "{disk_map}"),
            CompactResult::CompactedDiskMap(disk_map) => write!(f, "{disk_map}"),
        }
    }
}

fn inflate(blocks: Vec<CompactBlock>) -> DiskMap {
    let blocks = blocks
        .into_iter()
        .flat_map(|block| {
            match block {
                CompactBlock::File(FileBlock { id, span }) => {
                    vec![Some(id); span]
                },
                CompactBlock::FreeSpace(size) => vec![None; size],
            }
        })
        .collect::<Vec<_>>();
    DiskMap {
        blocks,
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct DeflatedDiskMap {
    blocks: Vec<CompactBlock>,
}

impl DeflatedDiskMap {
    fn inflate(self) -> DiskMap {
        inflate(self.blocks)
    }
}

impl std::fmt::Display for DeflatedDiskMap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        display_disk_map_blocks(f, &self.clone().inflate().blocks)
    }
}


#[derive(Debug, Clone, Default, PartialEq)]
struct DiskMap {
    blocks: Vec<Option<usize>>,
}

impl DiskMap {
    fn as_string(&self) -> String {
        let mut result = String::new();
        if let Err(err) = display_disk_map_blocks(&mut result, &self.blocks) {
            panic!("failed to display disk map: {}", err);
        }
        result
    }
    fn min_file_size(&self) -> usize {
        // TODO: we could optimize this by skipping based on last_file_pos and reversing the iterator
        let mut file_blocks = self.blocks.iter().enumerate().filter(|(_, b)| b.is_some()).peekable();
        let mut min_file_block_size = usize::MAX;
        while let Some((ix, Some(id))) = file_blocks.next() {
            let last_ix = ix;
            let mut file_block_size = 1;
            while let Some((ix, Some(next_id))) = file_blocks.peek() {
                if ix - last_ix > 1 || id != next_id {
                    break;
                }
                file_blocks.next();
                file_block_size += 1;
            }
            min_file_block_size = min_file_block_size.min(file_block_size);
        }
        min_file_block_size
    }
    fn max_free_space_size(&self) -> usize {
        // TODO: we could optimize this by skipping based on first_free_space_pos
        let mut free_space_blocks = self.blocks.iter().enumerate().filter(|(_, b)| b.is_none()).peekable();
        let mut max_free_space_size = 0;
        'outer: while let Some((ix, None)) = free_space_blocks.next() {
            let last_ix = ix;
            let mut free_space_size = 1;
            while let Some((ix, None)) = free_space_blocks.peek() {
                if ix - last_ix > 1 {
                    break;
                }
                free_space_blocks.next();
                free_space_size += 1;
                // don't count the last free space chunk since files cant be moved into it
                if *ix == self.blocks.len() - 1 {
                    break 'outer;
                }
            }
            max_free_space_size = max_free_space_size.max(free_space_size);
        }
        max_free_space_size
    }
    fn first_free_space_pos(&self) -> Option<usize> {
        self.blocks.iter().position(|b| b.is_none())
    }
    fn last_file_block_pos(&self) -> Option<usize> {
        self.blocks.iter().rposition(|b| b.is_some())
    }
    fn is_compact(&self) -> bool {
        let Some(first_free_space_pos) = self.first_free_space_pos() else {
            // no free space = compact
            println!("[is_compact]: no free space = compact");
            return true;
        };
        let Some(last_file_block_pos) = self.last_file_block_pos() else {
            // no file blocks = compact
            println!("[is_compact]: no file blocks = compact");
            return true
        };
        if first_free_space_pos > last_file_block_pos {
            // all free space occurs after file blocks = compact
            println!("[is_compact]: all free space occurs after file blocks = compact");
            return true
        }

        let max_free_space_size = self.max_free_space_size();
        let min_file_block_size = self.min_file_size();

        println!("[is_compact]: max_free_space_size: {max_free_space_size} < min_file_block_size: {min_file_block_size} = {}", max_free_space_size < min_file_block_size);
        // if false, we still have more compacting we can do
        max_free_space_size < min_file_block_size
    }
    fn _compact(&mut self, mut steps: Option<usize>) -> bool {
        println!("---\nstarting compact ({steps:?})\n---");
        println!("{}", self.as_string());
        let is_steps_completed = |steps: &Option<usize>| {
            if let Some(steps) = *steps {
                if steps == 0 {
                    return true;
                }
            }
            false
        };
        if is_steps_completed(&steps) {
            return false;
        }

        let total_blocks = self.blocks.len();
        let mut right_skip = 0;
        let mut swaps: Vec<(usize, usize)> = vec![];

        let mut iteration_map = HashMap::<&'static str, usize>::new();
        let mut check_iterations = |key: &'static str| {
            let entry = iteration_map.entry(key).or_default();
            if *entry > 1000 {
                panic!("max iterations!");
            }
            *entry += 1;
        };

        'commit: loop {
            check_iterations("loop start");
            {
                if !swaps.is_empty() {
                    println!("    before: {}", self.as_string());
                }
                for &(from_pos, to_pos) in swaps.iter() {
                    let Some(from) = self.blocks[from_pos].take() else {
                        panic!("attempted to swap from empty space!");
                    };
                    if self.blocks[to_pos].replace(from).is_some() {
                        panic!("attempted to swap to occupied space!");
                    }
                }
                if swaps.is_empty() {
                    println!("---\nrepeating search\n---");
                    // search files again to see if new room has been made
                    right_skip = 0;
                } else {
                    println!("     after: {}", self.as_string());
                    println!("---");
                }
                swaps.clear();
            }

            if is_steps_completed(&steps) {
                return self.is_compact();
            }

            let mut rblocks = self.blocks.iter().enumerate().rev().skip(right_skip).peekable();

            'file: while let Some((file_start_pos, block)) = rblocks.next() {
                check_iterations("file");
                let mut left_skip = 0;
                right_skip = total_blocks.saturating_sub(file_start_pos);
                let Some(id) = block else {
                    continue 'file;
                };
                let mut file_size = 1;
                'file_cont: while let Some(&(ix, Some(next_id))) = rblocks.peek() {
                    check_iterations("file");
                    if id != next_id {
                        break 'file_cont;
                    }
                    right_skip = total_blocks.saturating_sub(ix);
                    file_size += 1;
                    rblocks.next();
                }

                if left_skip > 0 && right_skip == left_skip {
                    println!("=== skip same! ===")
                }

                let file_end_pos = file_start_pos + file_size - 1;
                println!("---");
                println!("           file: {} @ {file_start_pos}..={file_end_pos}", vec![id.to_string();file_size].into_iter().collect::<String>());
                println!("     right_skip: {right_skip}");
                println!("      left_skip: {left_skip}");
                let mut lblocks = self.blocks.iter()
                    .enumerate()
                    // limit free space searched to before the current file block
                    .take(total_blocks.saturating_sub(right_skip))
                    .skip(left_skip)
                    .peekable();
                println!("lblocks to search: {}", lblocks.len());
                let mut last_seen_free_space_pos = None;
                'free_space: while let Some((free_space_start_pos, block)) = lblocks.next() {
                    check_iterations("free_space");
                    left_skip = free_space_start_pos;
                    let None = block else {
                        continue 'free_space;
                    };
                    let mut free_space = 1;
                    'free_space_cont: while let Some(&(ix, None)) = lblocks.peek() {
                        check_iterations("free_space");
                        free_space += 1;
                        lblocks.next();
                    }
                    let free_space_end_pos = free_space_start_pos + free_space - 1;
                    last_seen_free_space_pos = Some(free_space_end_pos);
                    println!("free_space: {} @ {free_space_start_pos}..={free_space_end_pos}", vec![".";free_space].into_iter().collect::<String>());
                    println!(" left_skip: {left_skip}");
                    if free_space >= file_size {
                        for n in 0..file_size {
                            swaps.push((file_start_pos - n, free_space_start_pos + n));
                        }
                        steps = steps.map(|x| x.saturating_sub(1));
                        continue 'commit;
                    }
                    continue 'free_space;
                }
                println!("---");
                if last_seen_free_space_pos.is_none() {
                    println!("no free space found before file");
                    println!("{}", self.as_string());
                    // if we didn't find any free_space between the current file and start of the disk
                    // restart search
                    right_skip = 0;

                    if self.is_compact() {
                        println!("compact! : {}", self.as_string());
                        println!();
                        println!();
                        println!();
                        return true;
                    }
                    continue 'commit;
                }
                continue 'file;
            }
        }
    }
    fn compact(mut self) -> CompactedDiskMap {
        let is_compacted = self._compact(None);
        if !is_compacted {
            panic!("failed to compact disk");
        }
        CompactedDiskMap {
            blocks: self.blocks
        }
    }
    fn compact_steps(mut self, steps: usize) -> CompactResult {
        if self._compact(Some(steps)) {
            CompactResult::CompactedDiskMap(CompactedDiskMap {
                blocks: self.blocks
            })
        } else {
            CompactResult::DiskMap(self)
        }
    }
}

impl std::fmt::Display for DiskMap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        display_disk_map_blocks(f, &self.blocks)
    }
}

impl TryFrom<&str> for DiskMap {
    type Error = DiskMapFromError;
    /// unchecked conversion of "0..1" to DiskMap.
    /// File id limited to 0-9
    fn try_from(value: &str) -> Result<Self, Self::Error> {

        enum ParsedBlock {
            FreeSpace,
            File(usize),
            Error(DiskMapFromError),
        }

        let parsed_blocks = value.chars()
            .enumerate()
            .map(|(pos, char)| {
                if char == '.' {
                    ParsedBlock::FreeSpace
                } else {
                    char.to_digit(10)
                        .map(|digit| ParsedBlock::File(digit as usize))
                        .unwrap_or_else(|| {
                            ParsedBlock::Error(DiskMapFromError::UnexpectedChar {
                                pos,
                                char,
                            })
                        })
                }
            });
        let mut blocks = vec![];
        for block in parsed_blocks.into_iter() {
            blocks.push(match block {
                ParsedBlock::FreeSpace => None,
                ParsedBlock::File(id) => Some(id),
                ParsedBlock::Error(err) => return Err(err),
            })
        }
        Ok(Self {
            blocks,
            ..Default::default()
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct CompactedDiskMap {
    blocks: Vec<Option<usize>>,
}

impl CompactedDiskMap {
    fn condensed(&self) -> Vec<CompactBlock> {
        let mut compact_blocks = vec![];
        let mut blocks = self.blocks.iter().peekable();
        while let Some(block) = blocks.next() {
            let compact_block = match *block {
                None => {
                    let mut free_space = 1;
                    while let Some(None) = blocks.peek() {
                        blocks.next();
                        free_space += 1;
                    }
                    CompactBlock::FreeSpace(free_space)
                }
                Some(id) => {
                    let mut block = FileBlock { id, span: 1 };
                    while let Some(&&Some(next_id)) = blocks.peek() {
                        if next_id != id {
                            break;
                        }
                        blocks.next();
                        block.span += 1;
                    }
                    CompactBlock::File(block)
                }
            };
            compact_blocks.push(compact_block);
        }
        compact_blocks
    }
    fn checksum(&self) -> usize {
        self.condensed().into_iter().enumerate().map(|(ix, block)| match block {
            CompactBlock::File(FileBlock { id, span: size }) => id * size * ix,
            CompactBlock::FreeSpace(_) => 0,
        }).sum()
    }
}

impl std::fmt::Display for CompactedDiskMap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        display_disk_map_blocks(f, &self.blocks)
    }
}

fn display_disk_map_blocks(f: &mut impl Write, blocks: &Vec<Option<usize>>) -> std::fmt::Result {
    for block in blocks.iter() {
        match block {
            Some(file_id) => write!(f, "{file_id}")?,
            None => write!(f, ".")?,
        }
    }
    Ok(())
}

#[derive(Error, Debug, Clone, Copy)]
enum DiskMapFromError {
    #[error("Unexpected char '{char}' at position {pos}")]
    UnexpectedChar {
        char: char,
        pos: usize,
    },
}

impl TryFrom<&str> for DeflatedDiskMap {
    type Error = DiskMapFromError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut blocks = vec![];
        let parts = value.chars()
            .enumerate()
            .map(|(pos, char)| {
                char.to_digit(10)
                    .map(|digit| digit as usize)
                    .ok_or_else(|| DiskMapFromError::UnexpectedChar {
                        pos,
                        char,
                    })
            });
        let mut id = 0;
        for mut chunk in &parts.into_iter().chunks(2) {
            let Some(file_size) = chunk.next() else { unreachable!() };
            blocks.push(CompactBlock::File(FileBlock { id, span: file_size? }));
            id += 1;
            let Some(free_space) = chunk.next() else { continue; };
            blocks.push(CompactBlock::FreeSpace(free_space?));
        };
        if blocks.last() == Some(&CompactBlock::FreeSpace(0)) {
            blocks.pop();
        }
        Ok(Self {
            blocks,
            ..Default::default()
        })
    }
}

impl TryFrom<String> for DeflatedDiskMap {
    type Error = DiskMapFromError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        DeflatedDiskMap::try_from(value.as_str())
    }
}

#[cfg(test)]
mod tests {
    use anyhow::anyhow;
    use super::*;

    #[test]
    fn test_inflate() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(DeflatedDiskMap::try_from("10")?.inflate().blocks, vec![Some(0)]);
        assert_eq!(DeflatedDiskMap::try_from("11")?.inflate().blocks, vec![Some(0), None]);
        assert_eq!(DeflatedDiskMap::try_from("121")?.inflate().blocks, vec![Some(0), None, None, Some(1)]);
        assert_eq!(DeflatedDiskMap::try_from("21")?.inflate().blocks, vec![Some(0), Some(0), None]);
        assert_eq!(DeflatedDiskMap::try_from("213")?.inflate().blocks, vec![Some(0), Some(0), None, Some(1), Some(1), Some(1)]);
        assert_eq!(
            DeflatedDiskMap::try_from("2333133121414131402")?.inflate().blocks,
            vec![
                [0;2].map(Some).to_vec(),
                [None; 3].to_vec(),
                [1;3].map(Some).to_vec(),
                [None; 3].to_vec(),
                [2;1].map(Some).to_vec(),
                [None; 3].to_vec(),
                [3;3].map(Some).to_vec(),
                [None; 1].to_vec(),
                [4;2].map(Some).to_vec(),
                [None; 1].to_vec(),
                [5;4].map(Some).to_vec(),
                [None; 1].to_vec(),
                [6;4].map(Some).to_vec(),
                [None; 1].to_vec(),
                [7;3].map(Some).to_vec(),
                [None; 1].to_vec(),
                [8;4].map(Some).to_vec(),
                [None; 0].to_vec(),
                [9;2].map(Some).to_vec(),
                [None; 0].to_vec(),
            ]
               .into_iter()
               .flatten()
               .collect::<Vec<_>>()
        );
        Ok(())
    }
    #[test]
    fn test_string_display() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("1")?.inflate()), "0");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("12")?.inflate()), "0..");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("121")?.inflate()), "0..1");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("12121")?.inflate()), "0..1..2");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("1212121")?.inflate()), "0..1..2..3");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("121212121")?.inflate()), "0..1..2..3..4");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("12121212121")?.inflate()), "0..1..2..3..4..5");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("1212121212121")?.inflate()), "0..1..2..3..4..5..6");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("121212121212121")?.inflate()), "0..1..2..3..4..5..6..7");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("12121212121212121")?.inflate()), "0..1..2..3..4..5..6..7..8");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("1212121212121212121")?.inflate()), "0..1..2..3..4..5..6..7..8..9");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("121212121212121212121")?.inflate()), "0..1..2..3..4..5..6..7..8..9..10");
        Ok(())
    }
    #[test]
    fn test_compact() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("1")?.inflate().compact()), "0");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("12")?.inflate().compact_steps(0)), "0..");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("12")?.inflate().compact_steps(1)), "0..");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("12")?.inflate().compact_steps(2)), "0..");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("12")?.inflate().compact()), "0..");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("121")?.inflate().compact()), "01..");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("2333133121414131402")?.inflate().compact()), "00992111777.44.333....5555.6666.....8888..");
        Ok(())
    }
    #[test]
    fn test_checksum() -> Result<(), Box<dyn std::error::Error>> {
        let disk_map = DeflatedDiskMap::try_from("2333133121414131402")?.inflate();
        println!("initial: {}", disk_map.as_string());
        let disk_map = disk_map.compact();
        assert_eq!(disk_map.checksum(), 2858);
        Ok(())
    }
    #[test]
    fn debug() -> anyhow::Result<()> {
        fn debug_log_disk_map_compacting(mut disk_map: DiskMap) -> anyhow::Result<CompactedDiskMap> {
            println!("initial: {}", disk_map);
            let free_space_count = disk_map.blocks.iter().filter(|b| b.is_none()).count();
            for step in 0..(free_space_count + 1) {
                let step = step + 1;
                // println!(" step {step}: {}", disk_map.clone().inflate());
                disk_map = match disk_map.compact_steps(1) {
                    CompactResult::DiskMap(disk_map) => disk_map,
                    CompactResult::CompactedDiskMap(disk_map) => return Ok(disk_map),
                }
            }

            Err(anyhow!("Failed to compact disk map: not enough steps"))
        }
        let disk_map = DeflatedDiskMap::try_from("2333133121414131402")?.inflate();
        let compacted_disk_map = debug_log_disk_map_compacting(disk_map)?;
        println!("compacted_disk_map: {compacted_disk_map}");
        assert_eq!(format!("{compacted_disk_map}"), "00992111777.44.333....5555.6666.....8888..");
        Ok(())
    }
    #[test]
    fn disk_map_try_from_str() -> anyhow::Result<()> {
        let input = "1..2.3...";
        assert_eq!(DiskMap::try_from(input)?.as_string(), input);
        let input = "1";
        assert_eq!(DiskMap::try_from(input)?.as_string(), input);
        let input = ".";
        assert_eq!(DiskMap::try_from(input)?.as_string(), input);
        let input = ".1";
        assert_eq!(DiskMap::try_from(input)?.as_string(), input);
        // DiskMap::try_from - file id limited to 0-9
        assert_eq!(DiskMap::try_from("10")?.blocks, vec![Some(1), Some(0)]);
        Ok(())
    }

    #[test]
    fn first_free_space_pos() -> anyhow::Result<()> {
        assert_eq!(DiskMap::try_from("1")?.first_free_space_pos(), None);
        assert_eq!(DiskMap::try_from(".")?.first_free_space_pos(), Some(0));
        assert_eq!(DiskMap::try_from("1.")?.first_free_space_pos(), Some(1));
        assert_eq!(DiskMap::try_from(".1")?.first_free_space_pos(), Some(0));
        assert_eq!(DiskMap::try_from("0.1")?.first_free_space_pos(), Some(1));
        assert_eq!(DiskMap::try_from("0..1")?.first_free_space_pos(), Some(1));
        assert_eq!(DiskMap::try_from("01..")?.first_free_space_pos(), Some(2));
        Ok(())
    }

    #[test]
    fn last_file_block_pos() -> anyhow::Result<()> {
        assert_eq!(DiskMap::try_from("1")?.last_file_block_pos(), Some(0));
        assert_eq!(DiskMap::try_from(".")?.last_file_block_pos(), None);
        assert_eq!(DiskMap::try_from("1.")?.last_file_block_pos(), Some(0));
        assert_eq!(DiskMap::try_from(".1")?.last_file_block_pos(), Some(1));
        assert_eq!(DiskMap::try_from("0.1")?.last_file_block_pos(), Some(2));
        assert_eq!(DiskMap::try_from("0..1")?.last_file_block_pos(), Some(3));
        Ok(())
    }

    #[test]
    fn is_compact() -> anyhow::Result<()> {
        assert!(DiskMap::try_from("1")?.is_compact());
        assert!(DiskMap::try_from(".")?.is_compact());
        assert!(DiskMap::try_from("1.")?.is_compact());
        assert!(!DiskMap::try_from(".1")?.is_compact());
        assert!(!DiskMap::try_from("0.1")?.is_compact());
        Ok(())
    }
}
