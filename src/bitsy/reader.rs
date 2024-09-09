use std::{any::type_name, cmp::min, convert::TryInto, prelude::rust_2021::TryFrom, rc::Rc};

use crate::{
    bitsy::parse_int,
    item::{
        info::{ItemDb, MapItemDb},
        properties::{MapPropertyDb, PropertyDb},
    },
};

use super::{
    bits_from_str,
    context::{ContextKey, ContextMap, ContextValue},
    error::{BitsyError, BitsyErrorKind},
    result::BitsyResult,
    BitReader, Bitsy, MyBitSlice, MyBitVec,
};

pub struct BitVecReader {
    bits: MyBitVec,
    index: usize,
    context: ContextMap,
    item_db: Rc<dyn ItemDb>,
}

fn int_to_printable_char(int: u32) -> char {
    match int {
        32..=126 | 161..=255 => char::from_u32(int).unwrap(),
        _ => '.',
    }
}

impl BitVecReader {
    pub fn dbless(bits: MyBitVec) -> Self {
        Self {
            bits,
            index: 0,
            context: ContextMap::new(),
            item_db: Rc::new(MapItemDb::new()),
        }
    }

    pub fn new<I: ItemDb + 'static>(bits: MyBitVec, item_db: I) -> Self {
        Self {
            bits,
            index: 0,
            context: ContextMap::new(),
            item_db: std::rc::Rc::new(item_db),
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

    fn queue_context_reset(&self) -> super::context::ContextResetGuard {
        self.context.context_reset()
    }

    fn set_context<T: ContextValue>(&mut self, key: &ContextKey<T>, value: T) {
        self.context.set_context(key, value);
    }

    fn get_context<T: ContextValue>(&self, key: &ContextKey<T>) -> BitsyResult<T> {
        self.context
            .get_context(key)
            .ok_or_else(|| self.error(BitsyErrorKind::MissingContext(key.as_str().to_string())))
    }

    fn item_db(&self) -> Rc<dyn ItemDb> {
        self.item_db.clone()
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
            if self.bits[self.index..self.index + padding].any() {
                return Err(self.error(BitsyErrorKind::InvalidData("Padding not zero".to_string())));
            }
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

    fn read_until(&mut self, bits: &MyBitVec) -> BitsyResult<MyBitVec> {
        let result = self
            .search(bits, 0)
            .map(|offset| self.bits[self.index..self.index + offset].to_owned())
            .unwrap_or_else(|| self.bits[self.index..].to_owned());
        self.index += result.len();
        Ok(result)
    }

    fn read_property_tail(&mut self) -> BitsyResult<MyBitVec> {
        let terminator = bits_from_str("111 111 111").unwrap();
        let mut match_index = self.index
            + self.search(&terminator, 0).ok_or_else(|| {
                self.error(BitsyErrorKind::InvalidData(
                    "Could not find property tail".to_string(),
                ))
            })?;
        while match_index + 9 < self.bits.len() && self.bits[match_index + 9] {
            match_index += 1;
        }

        let tail = self.bits[self.index..match_index].to_owned();
        self.index = match_index + 9;
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

    fn search(&self, needle: &MyBitVec, offset: usize) -> Option<usize> {
        let mut start = self.index + offset;
        while start < self.bits.len() {
            if self.bits[start..].starts_with(needle) {
                return Some(start - self.index);
            }
            start += 1;
        }
        None
    }
}
