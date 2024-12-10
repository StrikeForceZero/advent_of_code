/// quite the mess this has become

use itertools::{Itertools};
use thiserror::Error;
use advent_of_code::read_input;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = read_input(9);
    let line = input.next().expect("input empty")?;
    let disk_map = DeflatedDiskMap::try_from(line)?;
    assert!(input.next().is_none(), "input expected to be single line");
    // println!("Disk map: {disk_map}");
    let compact_disk_map = disk_map.compact();
    // println!("Compact Disk map: {compact_disk_map}");
    let checksum = compact_disk_map.checksum();
    println!("Checksum: {checksum}");
    Ok(())
}

#[derive(Debug, Clone, PartialEq)]
struct File {
    id: usize,
    size: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct FreeSpace {
    size: usize,
}

impl FreeSpace {
    fn can_fit(&self, file: &File) -> bool {
        self.size >= file.size
    }
    fn take_free_space(mut self, amount: usize) -> Option<FreeSpace> {
        self.size = self.size.saturating_sub(amount);
        if self.size == 0 {
            None
        } else {
            Some(self)
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum CompactBlock {
    File(File),
    FreeSpace(FreeSpace),
}

impl CompactBlock {
    fn can_fit(&self, file: &File) -> bool {
        match self {
            CompactBlock::File(_) => false,
            CompactBlock::FreeSpace(free_space) => free_space.can_fit(file),
        }
    }
    fn take_free_space(mut self, amount: usize) -> Option<FreeSpace> {
        if let CompactBlock::FreeSpace(free_space) = self {
            free_space.take_free_space(amount)
        } else {
            None
        }
    }
    fn take_file(&mut self) -> Option<File> {
        if let CompactBlock::File(file) = self {
            let file = file.clone();
            *self = CompactBlock::FreeSpace(FreeSpace { size: file.size });
            Some(file)
        } else {
            None
        }
    }
    fn file(self) -> Option<File> {
        match self {
            CompactBlock::File(file) => Some(file),
            CompactBlock::FreeSpace(_) => None,
        }
    }
    fn free_space(self) -> Option<FreeSpace> {
        match self {
            CompactBlock::File(_) => None,
            CompactBlock::FreeSpace(free_space) => Some(free_space),
        }
    }
    fn is_file(&self) -> bool {
        match self {
            CompactBlock::File(_) => true,
            CompactBlock::FreeSpace(_) => false,
        }
    }
    fn is_free_space(&self) -> bool {
        match self {
            CompactBlock::File(_) => false,
            CompactBlock::FreeSpace(_) => true,
        }
    }
    fn is_non_zero_free_space(&self) -> bool {
        match self {
            CompactBlock::File(_) => false,
            CompactBlock::FreeSpace(FreeSpace { size }) => *size > 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum CompactResult {
    DeflatedDiskMap(DeflatedDiskMap),
    CompactedDiskMap(CompactedDiskMap),
}

impl CompactResult {
    fn inflate(self) -> DiskMap {
        match self {
            CompactResult::DeflatedDiskMap(deflated_disk_map) => deflated_disk_map.inflate(),
            CompactResult::CompactedDiskMap(compacted_disk_map) => compacted_disk_map.inflate(),
        }

    }
}

impl std::fmt::Display for CompactResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CompactResult::DeflatedDiskMap(deflated_disk_map) => write!(f, "{deflated_disk_map}"),
            CompactResult::CompactedDiskMap(compacted_disk_map) => write!(f, "{compacted_disk_map}"),
        }
    }
}

fn inflate(blocks: Vec<CompactBlock>) -> DiskMap {
    let blocks = blocks
        .into_iter()
        .flat_map(|block| {
            match block {
                CompactBlock::File(File {id, size}) => {
                    vec![Some(id); size]
                },
                CompactBlock::FreeSpace(FreeSpace { size }) => vec![None; size],
            }
        })
        .collect();
    DiskMap {
        blocks,
    }
}



#[derive(Debug, Copy, Clone, Default, PartialEq)]
struct Seen {
    left: usize,
    right: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct DeflatedDiskMap {
    blocks: Vec<CompactBlock>,
    seen: Seen,
}

impl DeflatedDiskMap {
    fn inflate(self) -> DiskMap {
        inflate(self.blocks)
    }
    fn print_debug_string(&self) {
        for (ix, block) in self.blocks.iter().enumerate() {
            print!("[");
            match block {
                CompactBlock::File(File { id, size }) => {
                    let value = (0..*size).map(|_| id.to_string()).collect::<String>();
                    print!("{value:^size$}", size=size.max(&ix.to_string().len()));
                }
                CompactBlock::FreeSpace(FreeSpace { size }) => {
                    let value = (0..*size).map(|_| ".").collect::<String>();
                    print!("{value:^size$}", size=size.max(&ix.to_string().len()));
                }
            }
            print!("]");
        }
        println!();
        for (ix, block) in self.blocks.iter().enumerate() {
            print!("[");
            match block {
                CompactBlock::File(File { id, size }) => {
                    print!("{ix:^size$}", size=size.max(&ix.to_string().len()));
                }
                CompactBlock::FreeSpace(FreeSpace { size }) => {
                    print!("{ix:^size$}", size=size.max(&ix.to_string().len()));
                }
            }
            print!("]");
        }
        println!();
    }
    fn _compact(&mut self, mut steps: Option<usize>) -> bool {
        self.blocks.retain(|block| block.is_non_zero_free_space() || block.is_file());
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

        fn open_pos_iter<'a>(blocks: &'a Vec<CompactBlock>, seen: &'a mut Seen) -> impl Iterator<Item = usize> + 'a  {
            let seen_left = seen.left;
            blocks.iter().enumerate().skip(seen_left).map(|(ix, block)| { seen.left = ix + 1; block }).positions(CompactBlock::is_non_zero_free_space).map(move |pos| pos + seen_left)
        }
        fn file_pos_iter<'a>(blocks: &'a Vec<CompactBlock>, seen: &'a mut Seen) -> impl Iterator<Item = usize> + 'a  {
            let seen_right = seen.right;
            blocks.iter().rev().enumerate().skip(seen_right).map(|(ix, block)| { seen.right = ix + 1; block }).positions(CompactBlock::is_file).map(move |pos| blocks.len().saturating_sub(1) - pos - seen_right)
        }

        let mut ix = 0;
        'outer: loop {
            // println!("\nloop {:?}", self.seen);

            let mut blocks = self.blocks.clone();
            let mut removals = vec![];
            let mut skip = 0;
            for (ix, block) in self.blocks.iter_mut().enumerate() {
                skip += 1;
                match block {
                    CompactBlock::File(_) => {
                        continue;
                    }
                    CompactBlock::FreeSpace(ref mut file) => {
                        for (ix, block) in blocks.iter().enumerate().skip(skip) {
                            match block {
                                CompactBlock::File(_) => {
                                    break;
                                }
                                CompactBlock::FreeSpace(next_file) => {
                                    skip += 1;
                                    file.size += next_file.size;
                                    removals.push(ix);
                                }
                            }
                        }
                    }
                }
            }
            for removal in removals.into_iter().rev() {
                self.blocks.remove(removal);
            }

            let Some(file_pos) = file_pos_iter(&self.blocks, &mut self.seen).next() else { self.seen.right = 0; println!("files exhausted"); return false; };
            let mut cur_file = None;

            loop {
                ix += 1;
                if ix > 1000 {
                    panic!("too many iterations");
                }
                let Some(open_pos) = open_pos_iter(&self.blocks, &mut self.seen).next() else { self.seen.left = 0;
                    println!("free space exhausted");
                    if let Some(file) = cur_file.take() {
                        // println!("returning file to original position: {file_pos}");
                        self.blocks.insert(file_pos, CompactBlock::File(file));
                    }
                    continue 'outer;
                };

                // println!("open_pos: {open_pos}, file_pos: {file_pos}, len: {}", self.blocks.len());

                // if file_pos <= open_pos {
                //     println!("compact completed");
                //     return true;
                // }

                // println!("steps remaining: {steps:?}");
                if is_steps_completed(&steps) {
                    self.seen.left = self.seen.left.saturating_sub(1);
                    self.seen.right = self.seen.right.saturating_sub(1);
                    if let Some(file) = cur_file.take() {
                        // println!("returning file to original position: {file_pos}");
                        self.blocks.insert(file_pos, CompactBlock::File(file));
                    }
                    // println!("step completed\n");
                    println!("after:");
                    self.print_debug_string();
                    return false;
                }

                // println!();
                // self.print_debug_string();
                // println!("file_pos item: {:?}", self.blocks[file_pos]);
                // println!("open_pos item: {:?}", self.blocks[open_pos]);

                let file = if let Some(file) = cur_file.take() {
                    file
                } else {
                    let Some(file) = self.blocks.remove(file_pos).file() else { unreachable!() };
                    // println!("===== file: {file:?} ======");
                    println!("\n{}", (0..file.size).map(|_| file.id.to_string()).collect::<String>());
                    println!("before:");
                    self.print_debug_string();
                    file
                };

                let removed = self.blocks.remove(open_pos);
                let CompactBlock::FreeSpace(mut free_space) = removed else {
                    self.blocks.insert(open_pos, removed.clone());
                    println!();
                    self.print_debug_string();
                    panic!("{open_pos} not free {removed:?}")
                };

                // println!("checking {file:?} @ {file_pos} with {free_space:?} @ {open_pos}");
                if free_space.can_fit(&file) {
                    // println!("taking free_space for: {file:?} @ {open_pos}");
                    if let Some(remaining_free_space) = free_space.take_free_space(file.size) {
                        // println!("re-adding remaining free space: {remaining_free_space:?} @ {open_pos}");
                        self.blocks.insert(open_pos, CompactBlock::FreeSpace(remaining_free_space));
                    }
                    // self.seen.left = self.seen.left.saturating_sub(2);
                    // self.seen.right = self.seen.right.saturating_sub(1);
                    // println!("inserting file: {file:?} @ {open_pos}");
                    self.blocks.insert(file_pos, CompactBlock::FreeSpace(FreeSpace { size: file.size}));
                    self.blocks.insert(open_pos, CompactBlock::File(file));
                    steps = steps.map(|x| x.saturating_sub(1));
                } else {
                    // println!("can't fit");
                    self.blocks.insert(open_pos, CompactBlock::FreeSpace(free_space));
                    cur_file = Some(file);
                }
                // self.print_debug_string();
                // println!();
            }
        }
    }
    fn compact(mut self) -> CompactedDiskMap {
        if !self._compact(None) {
            unreachable!();
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
            CompactResult::DeflatedDiskMap(self)
        }
    }
}

impl std::fmt::Display for DeflatedDiskMap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        display_disk_map_blocks(&self.clone().inflate().blocks, f)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct DiskMap {
    blocks: Vec<Option<usize>>,
}

impl std::fmt::Display for DiskMap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        display_disk_map_blocks(&self.blocks, f)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct CompactedDiskMap {
    blocks: Vec<CompactBlock>,
}

impl CompactedDiskMap {
    fn inflate(self) -> DiskMap {
        inflate(self.blocks)
    }
    fn checksum(&self) -> usize {
        self.blocks.iter().enumerate().map(|(ix, block)| match block {
            CompactBlock::File(File { id, size }) => id * size * ix,
            CompactBlock::FreeSpace(_) => 0,
        }).sum()
    }
}

impl std::fmt::Display for CompactedDiskMap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ddm = DeflatedDiskMap {
            blocks: self.blocks.clone(),
            ..Default::default()
        };
        display_disk_map_blocks(&ddm.inflate().blocks, f)
    }
}

fn display_disk_map_blocks(blocks: &Vec<Option<usize>>, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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
            blocks.push(CompactBlock::File(File { id, size: file_size? }));
            id += 1;
            let Some(free_space) = chunk.next() else { continue; };
            blocks.push(CompactBlock::FreeSpace(FreeSpace { size: free_space? }));
        };
        if blocks.last() == Some(&CompactBlock::FreeSpace(FreeSpace::default())) {
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
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("1")?.compact().inflate()), "0");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("12")?.compact_steps(0).inflate()), "0..");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("12")?.compact_steps(1).inflate()), "0..");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("12")?.compact_steps(2).inflate()), "0..");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("12")?.compact().inflate()), "0..");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("121")?.compact().inflate()), "01..");
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("2333133121414131402")?.compact().inflate()), "00992111777.44.333....5555.6666.....8888..");
        Ok(())
    }
    #[test]
    fn test_checksum() -> Result<(), Box<dyn std::error::Error>> {
        let disk_map = DeflatedDiskMap::try_from("2333133121414131402")?.compact();
        assert_eq!(disk_map.checksum(), 2858);
        Ok(())
    }
    #[test]
    fn debug() -> anyhow::Result<()> {
        fn debug_log_disk_map_compacting(mut disk_map: DeflatedDiskMap) -> anyhow::Result<CompactedDiskMap> {
            println!("initial: {}", disk_map.clone().inflate());
            let free_space_count = disk_map.blocks.iter().filter(|b| CompactBlock::is_free_space(b)).count();
            for step in 0..(free_space_count + 1) {
                let step = step + 1;
                // println!(" step {step}: {}", disk_map.clone().inflate());
                disk_map = match disk_map.compact_steps(1) {
                    CompactResult::DeflatedDiskMap(disk_map) => disk_map,
                    CompactResult::CompactedDiskMap(disk_map) => return Ok(disk_map),
                }
            }

            Err(anyhow!("Failed to compact disk map: not enough steps"))
        }
        let disk_map = DeflatedDiskMap::try_from("2333133121414131402")?;
        let compacted_disk_map = debug_log_disk_map_compacting(disk_map)?;
        println!("compacted_disk_map: {compacted_disk_map}");
        assert_eq!(format!("{compacted_disk_map}"), "00992111777.44.333....5555.6666.....8888..");
        Ok(())
    }
}
