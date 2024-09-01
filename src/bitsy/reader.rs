use std::{any::type_name, cmp::min, convert::TryInto, prelude::rust_2021::TryFrom};

use crate::item::{
    info::{ItemDb, MapItemDb},
    properties::{MapPropertyDb, PropertyDb},
};

use super::{
    context::{ContextKey, ContextMap, ContextStore, ContextValue},
    error::{BitsyError, BitsyErrorKind},
    result::BitsyResult,
    BitReader, Bitsy, MyBitSlice, MyBitVec,
};

pub struct BitVecReader {
    bits: MyBitVec,
    index: usize,
    context: ContextMap,
}

fn parse_int(bits: &MyBitSlice) -> Result<u32, String> {
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

fn int_to_printable_char(int: u32) -> char {
    match int {
        32..=126 | 161..=255 => char::from_u32(int).unwrap(),
        _ => '.',
    }
}

impl BitVecReader {
    pub fn new(bits: MyBitVec) -> Self {
        Self {
            bits,
            index: 0,
            context: ContextMap::new(),
        }
    }

    fn error(&self, kind: BitsyErrorKind) -> BitsyError {
        BitsyError::new(kind, self.index)
    }
}

impl BitReader for BitVecReader {
    fn index(&self) -> usize {
        self.index
    }

    fn get_context<T: ContextValue>(&self, key: &ContextKey<T>) -> BitsyResult<T> {
        self.context
            .get_context(key)
            .ok_or_else(|| self.error(BitsyErrorKind::MissingContext(key.as_str().to_string())))
    }

    fn set_context<T: ContextValue>(&mut self, key: &ContextKey<T>, value: T) {
        self.context.set_context(key, value)
    }

    fn item_db(&self) -> impl ItemDb {
        MapItemDb::new()
    }

    fn property_db(&self) -> impl PropertyDb {
        MapPropertyDb::new()
    }

    fn read_int<T: TryFrom<u32>>(&mut self, bit_count: usize) -> BitsyResult<T> {
        if bit_count > 32 {
            return Err(self.error(BitsyErrorKind::InvalidData(
                "Ints > 32 bits not supported".to_string(),
            )));
        }
        if self.index + bit_count > self.bits.len() {
            return Err(self.error(BitsyErrorKind::EndOfData));
        }

        let mut res: u32 = 0;
        let mut multiplier = 1;

        for index in 0..bit_count {
            res += self.bits[self.index + index] as u32 * multiplier;
            if index < 31 {
                multiplier *= 2;
            }
        }

        let result = T::try_from(res).map_err(|_| {
            self.error(BitsyErrorKind::InvalidData(format!(
                "Could not fit int of {} bits in {}",
                bit_count,
                type_name::<T>()
            )))
        })?;
        self.index += bit_count;
        Ok(result)
    }

    fn read_bits(&mut self, bit_count: usize) -> BitsyResult<MyBitVec> {
        let mut bitvec = MyBitVec::new();
        for bit in self.bits[self.index..].iter().take(bit_count) {
            bitvec.push(*bit);
        }
        if bitvec.len() != bit_count {
            return Err(self.error(BitsyErrorKind::EndOfData));
        }
        self.index += bit_count;
        Ok(bitvec)
    }

    fn read_padding(&mut self) -> BitsyResult<()> {
        if self.index % 8 != 0 {
            let padding = 8 - (self.index % 8);
            // read_padding does not fail if there is not enough data
            self.index = min(self.index + padding, self.bits.len());
        }
        Ok(())
    }

    fn read_tail(&mut self) -> BitsyResult<MyBitVec> {
        let mut tail = MyBitVec::new();
        tail.extend_from_bitslice(&self.bits[self.index..]);
        self.index = self.bits.len();
        Ok(tail)
    }

    fn report_next_bytes(&self, count: usize) {
        println!(
            "BitReader status: in bit {} (byte {}) of {} ({} bytes)",
            self.index,
            self.index / 8,
            self.bits.len(),
            self.bits.len() / 8
        );

        for byte_index in 0..count {
            let byte_start_index = self.index + byte_index * 8;
            let byte_end_index = min(byte_start_index + 8, self.bits.len());
            let bits = &self.bits[byte_start_index..byte_end_index];
            let value = parse_int(bits).unwrap();
            let bit_string = bits.to_string().replace(", ", "");
            if bits.len() < 8 {
                if !bits.is_empty() {
                    println!("Byte #{byte_index:4}: {bit_string} ({value}) (partial byte)",);
                }
                break;
            } else {
                println!(
                    "Byte #{byte_index:04}: {bit_string} {value:3} 0x{value:02X} ({})",
                    int_to_printable_char(value)
                );
            }
        }
    }

    fn peek<T: Bitsy>(&mut self) -> BitsyResult<T> {
        let index = self.index;
        let context = self.context.clone();
        let value = T::parse(self);
        self.index = index;
        self.context = context;
        value
    }
}
