pub mod context;
pub mod error;
mod huffman;
pub mod impls;
pub mod macros;
mod old;
mod reader;
pub mod result;
pub mod structs;
mod writer;

use std::{
    cmp::min,
    convert::{TryFrom, TryInto},
    fmt::Debug,
    rc::Rc,
};

use context::{ContextKey, ContextResetGuard, ContextValue};
pub use huffman::{HuffmanChar, HuffmanChars};
pub use old::*;
pub use reader::BitVecReader;
pub use writer::BitVecWriter;

use result::BitsyResult;

use crate::item::{info::ItemDb, properties::PropertyDb};

pub fn parse_int(bits: &MyBitSlice) -> Result<u32, String> {
    if bits.len() > 32 {
        return Err("Ints > 32 bits not supported".to_string());
    }
    let mut res: u32 = 0;
    let mut multiplier = 1u64;

    for bit in bits {
        if *bit {
            let multiplier: u32 = multiplier
                .try_into()
                .map_err(|_| "Multiplier was too big, should not happen".to_string())?;
            res += multiplier
        }
        multiplier *= 2;
    }

    Ok(res)
}

pub trait BitReader: Sized {
    fn index(&self) -> usize;

    fn queue_context_reset(&self) -> ContextResetGuard;
    fn get_context<T: ContextValue>(&self, key: &ContextKey<T>) -> BitsyResult<T>;
    fn set_context<T: ContextValue>(&mut self, key: &ContextKey<T>, value: T);

    fn item_db(&self) -> Rc<dyn ItemDb>;
    fn property_db(&self) -> impl PropertyDb;

    fn read_int<T: TryFrom<u32>>(&mut self, bit_count: usize) -> BitsyResult<T>;
    fn read_bits(&mut self, bit_count: usize) -> BitsyResult<MyBitVec>;
    fn read_padding(&mut self) -> BitsyResult<()>;
    fn read_tail(&mut self) -> BitsyResult<MyBitVec>;
    fn read_until(&mut self, bits: &MyBitVec) -> BitsyResult<MyBitVec>;
    fn read_property_tail(&mut self) -> BitsyResult<MyBitVec>;

    fn read<T: Bitsy>(&mut self) -> BitsyResult<T> {
        T::parse(self)
    }
    fn peek<T: Bitsy>(&mut self) -> BitsyResult<T>;
    fn search(&self, needle: &MyBitVec, offset: usize) -> Option<usize>;

    fn report_next_bytes(&self, count: usize);
    fn report_search(&self, needle: &MyBitVec) {
        println!(
            "BitReader status: in bit {} (byte {})",
            self.index(),
            self.index() / 8,
        );
        println!("Searching for: {needle}");
        let mut match_count = 0;
        let mut offset = 0;
        while let Some(find_offset) = self.search(needle, offset) {
            match_count += 1;
            println!(
                "Found match #{match_count} at offset {find_offset} (index {})",
                self.index() + find_offset
            );
            offset = find_offset + 1;
        }
        if match_count == 0 {
            println!("No matches found");
        }
    }
}

pub trait BitWriter: Sized {
    fn index(&self) -> usize;

    fn queue_context_reset(&self) -> ContextResetGuard;
    fn get_context<T: ContextValue>(&self, key: &ContextKey<T>) -> BitsyResult<T>;
    fn set_context<T: ContextValue>(&mut self, key: &ContextKey<T>, value: T);

    fn write_int<T: Into<u32>>(&mut self, value: T, bit_count: usize) -> BitsyResult<()>;
    fn write_bits(&mut self, value: &MyBitVec) -> BitsyResult<()>;
    fn write_padding(&mut self) -> BitsyResult<()>;

    fn write<T: Bitsy>(&mut self, value: &T) -> BitsyResult<()> {
        value.write_to(self)
    }
}

pub trait Bitsy: Sized + Debug {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self>;
    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()>;
}

pub trait BitSized {
    fn bit_size(&self) -> usize;
}

pub fn bitsy_to_bits(bitsy: &impl Bitsy, version: u32) -> MyBitVec {
    let mut writer = BitVecWriter::new(version);
    bitsy.write_to(&mut writer).unwrap();
    writer.into_bits()
}

pub fn compare_bitslices(
    expected: &MyBitSlice,
    actual: &MyBitSlice,
) -> Result<(), BitsliceCompareError> {
    let first_difference = find_first_difference(expected, actual);

    if expected.len() != actual.len() {
        let first = first_difference.unwrap_or(min(expected.len(), actual.len()));
        Err(BitsliceCompareError {
                expected:  show_bitslice_around(expected, first),
                actual:    show_bitslice_around(actual, first),
                message:               format!(
                "BitSlices sizes differ! Expected {} ({} bytes) bits, got {} ({} bytes). First difference at index {}",
                expected.len(),
                expected.len() / 8,
                actual.len(),
                actual.len() / 8,
                first
            )})
    } else if let Some(first_difference) = first_difference {
        Err(BitsliceCompareError {
            expected: show_bitslice_around(expected, first_difference),
            actual: show_bitslice_around(actual, first_difference),
            message: format!(
                "BitSlices differ! First difference at index {}",
                first_difference
            ),
        })
    } else {
        Ok(())
    }
}

pub struct BitsliceCompareError {
    expected: String,
    actual: String,
    message: String,
}

impl Debug for BitsliceCompareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Bitslices differ!: {}", self.message)?;
        writeln!(f, "Expected: {}", self.expected)?;
        writeln!(f, "Actual:   {}", self.actual)?;
        Ok(())
    }
}

fn find_first_difference(expected: &MyBitSlice, actual: &MyBitSlice) -> Option<usize> {
    let mut index = 0;
    while index < expected.len() && index < actual.len() {
        if expected[index] != actual[index] {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn bit_at(bits: &MyBitSlice, index: usize) -> char {
    if index < bits.len() {
        if bits[index] {
            '1'
        } else {
            '0'
        }
    } else {
        '.'
    }
}

fn as_bits(bits: &MyBitSlice, start: usize, end: usize) -> String {
    let mut result = String::new();
    for index in start..end {
        result.push(bit_at(bits, index));
    }
    result
}

const DIFF_WINDOW: usize = 16;

fn show_bitslice_around(bits: &MyBitSlice, index: usize) -> String {
    let mut result = String::new();
    if index > DIFF_WINDOW {
        result.push_str(&format!(" ({} hidden bits) ... ", index - DIFF_WINDOW));
    }

    let window_start = if index > DIFF_WINDOW {
        index - DIFF_WINDOW
    } else {
        0
    };
    result.push_str(&as_bits(bits, window_start, index));

    result.push('[');
    result.push(bit_at(bits, index));
    result.push(']');

    result.push_str(&as_bits(bits, index + 1, index + DIFF_WINDOW + 1));

    if index + DIFF_WINDOW + 1 < bits.len() {
        result.push_str(&format!(
            " ... ({} hidden bits)",
            bits.len() - index - DIFF_WINDOW
        ));
    }

    result
}

#[cfg(test)]
pub mod testutils {
    use std::cmp::min;

    use super::*;

    pub fn random_bits(bit_count: usize) -> MyBitVec {
        let mut bits = MyBitVec::new();
        for _index in 0..bit_count {
            bits.push(rand::random::<bool>());
        }
        bits
    }

    pub fn assert_reads_to<T: Bitsy + Eq>(bits: MyBitVec, expected: T) {
        let mut reader = BitVecReader::dbless(bits);
        let actual = reader.read().unwrap();
        assert_eq!(expected, actual);
        assert!(reader.read_tail().unwrap().is_empty());
    }

    pub fn bits<S: AsRef<str>>(bits: S) -> MyBitVec {
        bits_from_str(bits).unwrap()
    }
}
