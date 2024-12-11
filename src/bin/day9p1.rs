use itertools::{Itertools};
use thiserror::Error;
use advent_of_code::read_input_lines;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = read_input_lines(9);
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

#[derive(Debug, Clone, Copy, Default)]
struct FileId(usize);
impl FileId {
    fn next(self) -> Self {
        FileId(self.0 + 1)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum CompactBlock {
    File(usize),
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
            CompactResult::CompactedDiskMap(compacted_disk_map) => write!(f, "{compacted_disk_map}"),
        }
    }
}

#[derive(Debug, Clone, Default)]
struct DeflatedDiskMap {
    blocks: Vec<CompactBlock>,
}

impl DeflatedDiskMap {
    fn inflate(self) -> DiskMap {
        let mut file_id_iter = FileId::default();
        let blocks = self.blocks
            .into_iter()
            .flat_map(|block| {
                match block {
                    CompactBlock::File(size) => {
                        let FileId(file_id) = file_id_iter;
                        let file_blocks = vec![Some(file_id); size as usize];
                        file_id_iter = file_id_iter.next();
                        file_blocks
                    },
                    CompactBlock::FreeSpace(size) => vec![None; size as usize],
                }
            })
            .collect();
        DiskMap {
            blocks,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct DiskMap {
    blocks: Vec<Option<usize>>,
}

impl DiskMap {
    fn _compact(&mut self, mut steps: Option<usize>) -> bool {
        if let Some(steps) = steps {
            if steps == 0 {
                return false;
            }
        }
        let open_pos = self.blocks.iter().positions(Option::is_none);
        let take_pos = self.blocks.iter().positions(Option::is_some).rev();
        for (open_pos, taken_pos) in open_pos.zip(take_pos).collect::<Vec<_>>() {
            if taken_pos <= open_pos {
                return true;
            }
            let Some(taken) = self.blocks[taken_pos].take() else {
                unreachable!();
            };
            let replaced = self.blocks[open_pos].replace(taken);
            if replaced.is_some() {
                unreachable!();
            }
            steps = steps.map(|x| x.saturating_sub(1));
            if let Some(steps) = steps {
                if steps == 0 {
                    break;
                }
            }
        }
        // TODO: this kills performance for single step compacting
        //  but is required for the edge case where blocks might not end with free space
        if self.blocks.iter().rfind(|x| x.is_none()).is_none() {
            true
        } else {
            false
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
            CompactResult::DiskMap(self)
        }
    }
}

impl std::fmt::Display for DiskMap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        display_disk_map_blocks(&self.blocks, f)
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
struct CompactedDiskMap {
    blocks: Vec<Option<usize>>,
}

impl CompactedDiskMap {
    fn checksum(&self) -> usize {
        self.blocks.iter().enumerate().map(|(ix, block)| block.map_or(0, |x| x as usize * ix)).sum()
    }
}

impl std::fmt::Display for CompactedDiskMap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        display_disk_map_blocks(&self.blocks, f)
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
        for mut chunk in &parts.into_iter().chunks(2) {
            let Some(file_size) = chunk.next() else { unreachable!() };
            blocks.push(CompactBlock::File(file_size?));
            let Some(free_space) = chunk.next() else { continue; };
            blocks.push(CompactBlock::FreeSpace(free_space?));
        };
        if blocks.last() == Some(&CompactBlock::FreeSpace(0)) {
            blocks.pop();
        }
        Ok(Self {
            blocks,
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
        assert_eq!(format!("{}", DeflatedDiskMap::try_from("2333133121414131402")?.inflate().compact()), "0099811188827773336446555566..............");
        Ok(())
    }
    #[test]
    fn test_checksum() -> Result<(), Box<dyn std::error::Error>> {
        let disk_map = DeflatedDiskMap::try_from("2333133121414131402")?.inflate().compact();
        assert_eq!(disk_map.checksum(), 1928);
        Ok(())
    }
    #[test]
    fn debug() -> anyhow::Result<()> {
        fn debug_log_disk_map_compacting() -> anyhow::Result<CompactedDiskMap> {
            let mut disk_map = DeflatedDiskMap::try_from("2333133121414131402")?.inflate();

            let free_space_count = disk_map.blocks.iter().filter(|x| x.is_none()).count();
            for _ in 0..(free_space_count + 1) {
                println!("{disk_map}");
                disk_map = match disk_map.compact_steps(1) {
                    CompactResult::DiskMap(disk_map) => disk_map,
                    CompactResult::CompactedDiskMap(disk_map) => return Ok(disk_map),
                }
            }

            Err(anyhow!("Failed to compact disk map: not enough steps"))
        }
        let compacted_disk_map = debug_log_disk_map_compacting()?;
        println!("{compacted_disk_map}");
        assert_eq!(format!("{compacted_disk_map}"), "0099811188827773336446555566..............");
        Ok(())
    }
}
