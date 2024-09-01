pub mod context;
pub mod error;
mod huffman;
pub mod impls;
pub mod macros;
mod old;
mod reader;
pub mod result;
mod writer;

use std::convert::TryFrom;

use context::{ContextKey, ContextValue};
pub use huffman::HuffmanChars;
pub use old::*;
pub use reader::BitVecReader;
pub use writer::BitVecWriter;

use result::BitsyResult;

use crate::item::{info::ItemDb, properties::PropertyDb};

pub trait BitReader: Sized {
    fn index(&self) -> usize;

    fn get_context<T: ContextValue>(&self, key: &ContextKey<T>) -> BitsyResult<T>;
    fn set_context<T: ContextValue>(&mut self, key: &ContextKey<T>, value: T);

    fn item_db(&self) -> impl ItemDb;
    fn property_db(&self) -> impl PropertyDb;

    fn read_int<T: TryFrom<u32>>(&mut self, bit_count: usize) -> BitsyResult<T>;
    fn read_bits(&mut self, bit_count: usize) -> BitsyResult<MyBitVec>;
    fn read_padding(&mut self) -> BitsyResult<()>;
    fn read_tail(&mut self) -> BitsyResult<MyBitVec>;

    fn read<T: Bitsy>(&mut self) -> BitsyResult<T> {
        T::parse(self)
    }
    fn peek<T: Bitsy>(&mut self) -> BitsyResult<T>;

    fn report_next_bytes(&self, count: usize);
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

pub trait Bitsy: Sized {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self>;
    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()>;
}

pub trait BitSized {
    fn bit_size(&self) -> usize;
}
