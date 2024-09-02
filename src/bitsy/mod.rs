pub mod context;
pub mod error;
mod huffman;
pub mod impls;
pub mod macros;
mod old;
mod reader;
pub mod result;
mod writer;

use std::{convert::TryFrom, fmt::Debug};

use context::{ContextKey, ContextResetGuard, ContextValue};
pub use huffman::HuffmanChars;
pub use old::*;
pub use reader::BitVecReader;
pub use writer::BitVecWriter;

use result::BitsyResult;

use crate::item::{info::ItemDb, properties::PropertyDb};

pub trait BitReader: Sized {
    fn index(&self) -> usize;

    fn queue_context_reset(&self) -> ContextResetGuard;
    fn get_context<T: ContextValue>(&self, key: &ContextKey<T>) -> BitsyResult<T>;
    fn set_context<T: ContextValue>(&mut self, key: &ContextKey<T>, value: T);

    fn item_db(&self) -> impl ItemDb;
    fn property_db(&self) -> impl PropertyDb;

    fn read_int<T: TryFrom<u32>>(&mut self, bit_count: usize) -> BitsyResult<T>;
    fn read_bits(&mut self, bit_count: usize) -> BitsyResult<MyBitVec>;
    fn read_padding(&mut self) -> BitsyResult<()>;
    fn read_tail(&mut self) -> BitsyResult<MyBitVec>;
    fn read_until(&mut self, bits: &MyBitVec) -> BitsyResult<MyBitVec>;

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
        println!("Searching for: {needle:?}");
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
    fn version(&self) -> Option<u32>;

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
