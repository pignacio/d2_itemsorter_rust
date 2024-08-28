use std::cmp::min;
use std::convert::TryFrom;
use std::fmt::Display;

use bitvec::prelude::*;
use bitvec::view::BitViewSized;

pub type MyBitOrder = Lsb0;
pub type MyBitVec = BitVec<MyBitOrder, u8>;

pub fn bit_view(bit_vec: &MyBitVec, start: usize, size: usize) -> String {
    bit_vec[start..std::cmp::min(start + size, bit_vec.len())]
        .to_string()
        .replace(", ", "")
}

pub struct BitVecReader {
    bytes: Vec<u8>,
    bits: MyBitVec,
    index: usize,
}

impl Display for BitVecReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BitReader(@bit {} of {}, byte {} of {})",
            self.index,
            self.bits.len(),
            self.index / 8,
            self.bytes.len()
        )
    }
}

impl BitVecReader {
    pub fn new(bytes: Vec<u8>) -> BitVecReader {
        BitVecReader {
            bytes: bytes.to_vec(),
            bits: BitVec::from_vec(bytes),
            index: 0,
        }
    }

    pub fn get(&self, index: usize) -> bool {
        self.bits[index]
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn len(&self) -> usize {
        self.bits.len()
    }

    pub fn peek_bits(&self, size: usize) -> String {
        bit_view(&self.bits, self.index, size)
    }

    pub fn read_optional_byte_arr<const N: usize>(&mut self) -> Option<[u8; N]> {
        let is_present = self.read_bool();
        if is_present {
            Some(self.read_byte_arr())
        } else {
            None
        }
    }
    //
    // fn read_char_arr<const N: usize>(&mut self) -> [char; N] {
    //     let bytes: [u8; N] = self.read_byte_arr();
    //
    //     let mut chars = [char; N];
    //
    //     for i in 0..N {
    //         chars[i] = bytes[i] as char;
    //     }
    //
    //     return chars;
    // }

    pub fn read_into_bitarr<T: BitViewSized>(
        &mut self,
        size: usize,
        array: &mut BitArray<MyBitOrder, T>,
    ) {
        for (index, bit) in self.bits[self.index..].iter().enumerate().take(size) {
            array.set(index, *bit);
        }
        self.index += size;
    }

    pub fn read_optional_bits(&mut self, num_bits: usize) -> Option<MyBitVec> {
        let is_present = self.read_bool();
        if is_present {
            Some(self.read_bits(num_bits))
        } else {
            None
        }
    }

    pub fn read_bool(&mut self) -> bool {
        self.read_int::<u32>(1) != 0
    }

    pub fn read_optional_int<T: TryFrom<u32>>(&mut self, num_bits: usize) -> Option<T> {
        let is_present = self.read_bool();
        if is_present {
            Some(self.read_int(num_bits))
        } else {
            None
        }
    }

    fn find_match_index(&self, sentinel: &[u8]) -> Option<usize> {
        let byte_start_index = (self.index - 1) / 8 + 1;
        // println!("Searching for match: {} from index: {} [bit:{}]", arr_to_str(sentinel), byte_start_index, self.index);
        let mut current_match_size: usize = 0;
        for index in byte_start_index..self.bytes.len() {
            let byte = self.bytes[index];
            if byte == sentinel[current_match_size] {
                // println!("Partial match! index:{}, current match:{}, next byte:{}", index, current_match_size, self.bytes[index + 1]);
                current_match_size += 1;
                if current_match_size == sentinel.len() {
                    return Some(index - sentinel.len() + 1);
                }
            } else {
                current_match_size = 0;
            }
        }
        None
    }

    fn find_bits_match_index(&self, sentinel: &MyBitVec) -> Option<usize> {
        let bit_start_index = self.index;
        // println!("Searching for match: {} from index: {} [bit:{}]", arr_to_str(sentinel), byte_start_index, self.index);
        let mut current_match_size: usize = 0;
        for index in bit_start_index..self.bits.len() {
            let bit = self.get(index);
            if bit == sentinel[current_match_size] {
                // println!("Partial match! index:{}, current match:{}, next byte:{}", index, current_match_size, self.bytes[index + 1]);
                current_match_size += 1;
                if current_match_size == sentinel.len() {
                    return Some(index - sentinel.len() + 1);
                }
            } else {
                current_match_size = 0;
            }
        }
        None
    }
}

pub trait BitReader {
    fn read_byte_arr<const N: usize>(&mut self) -> [u8; N];
    fn read_int<T: TryFrom<u32>>(&mut self, num_bits: usize) -> T;
    fn read_bits(&mut self, size: usize) -> MyBitVec;
    fn read_padding(&mut self);
    fn read_until(&mut self, sentinel: &[u8]) -> MyBitVec;
    fn read_until_bits(&mut self, sentinel: &MyBitVec) -> MyBitVec;
    fn tail(&mut self) -> MyBitVec;
    fn read_versioned<T: VersionedBitsy>(&mut self, version: u32) -> T;
    fn read<T: Bitsy>(&mut self) -> T;

    fn report_next_bytes(&self, num_bytes: usize);
}

impl BitReader for BitVecReader {
    fn read_byte_arr<const N: usize>(&mut self) -> [u8; N] {
        let mut result = [0; N];

        result.iter_mut().for_each(|x| *x = self.read_int(8));
        result
    }

    fn read_int<T: TryFrom<u32>>(&mut self, num_bits: usize) -> T {
        assert!(num_bits <= 32, "Support for ints > 32 bits missing");
        let max_bits = std::mem::size_of::<T>() * 8;
        assert!(
            num_bits <= max_bits,
            "Type {} cannot hold {} bits. Max: {}",
            std::any::type_name::<T>(),
            num_bits,
            max_bits
        );
        let mut res: u32 = 0;
        let mut multiplier = 1;

        for index in 0..num_bits {
            res += self.bits[self.index + index] as u32 * multiplier;
            if index < 31 {
                multiplier *= 2;
            }
        }

        self.index += num_bits;
        T::try_from(res).unwrap_or_else(|_| panic!("Int did not fit"))
    }

    fn read_bits(&mut self, size: usize) -> MyBitVec {
        let mut bitvec = BitVec::new();
        for bit in self.bits[self.index..].iter().take(size) {
            bitvec.push(*bit);
        }
        self.index += size;
        bitvec
    }

    fn read_until(&mut self, sentinel: &[u8]) -> MyBitVec {
        let start = self.index;

        match self.find_match_index(sentinel) {
            Some(index) => {
                self.index = index * 8;
                // println!(
                //     "Found match at byte index: {}, new bit index = {}",
                //     index, self.index
                // );
                self.bits[start..self.index].to_bitvec()
            }
            _ => {
                self.index = self.bits.len();
                self.bits[start..].to_bitvec()
            }
        }
    }

    fn read_until_bits(&mut self, sentinel: &MyBitVec) -> MyBitVec {
        let start = self.index;
        match self.find_bits_match_index(sentinel) {
            Some(index) => {
                self.index = index;
                self.bits[start..self.index].to_bitvec()
            }
            _ => {
                self.bits.len();
                self.bits[start..].to_bitvec()
            }
        }
    }

    fn read_padding(&mut self) {
        let size = (8 - self.index % 8) % 8;
        let padding = self.read_bits(size);
        if padding.any() {
            panic!("Padding@{} is not zero: {}", self.index - size, padding);
        }
    }

    fn tail(&mut self) -> MyBitVec {
        self.bits[self.index..].to_bitvec()
    }

    fn read_versioned<T: VersionedBitsy>(&mut self, version: u32) -> T {
        T::parse(self, version)
    }

    fn read<T: Bitsy>(&mut self) -> T {
        T::parse(self)
    }

    fn report_next_bytes(&self, num_bytes: usize) {
        println!("Report of {}", self);
        if self.index % 8 != 0 {
            println!(
                "Unfinished byte @ {}: {}",
                self.index,
                self.peek_bits(8 - self.index % 8)
            );
        }
        let start_byte = self.index / 8 + if self.index % 8 == 0 { 0 } else { 1 };
        let end_byte = min(start_byte + num_bytes, self.bytes.len());
        for index in start_byte..end_byte {
            let byte = self.bytes[index];
            println!(
                "Byte@{}: {:3} 0x{:02X} {}",
                index,
                byte,
                byte,
                &self.bits[index * 8..index * 8 + 8]
            );
            //lala
        }
    }
}

pub trait BitWriter: Sized {
    fn append_u32(&mut self, value: u32);
    fn append_u16(&mut self, value: u16);
    fn append_bool(&mut self, value: bool);
    fn append_int<T: Into<u32>>(&mut self, value: T, num_bits: usize);
    fn append_optional_int<T: Into<u32>>(&mut self, optional: Option<T>, num_bits: usize);
    fn append_bits(&mut self, bitvec: &MyBitVec);
    fn append_optional_bits(&mut self, optional: &Option<MyBitVec>);
    fn append_optional_byte_arr<const N: usize>(&mut self, optional: &Option<[u8; N]>);
    fn append_byte_arr<const N: usize>(&mut self, array: &[u8; N]);
    fn append_padding(&mut self);

    fn append<T: BitWritable>(&mut self, value: &T) {
        value.append_to(self);
    }
}

impl BitWriter for MyBitVec {
    fn append_u32(&mut self, value: u32) {
        self.append_int(value, 32);
    }

    fn append_u16(&mut self, value: u16) {
        self.append_int(value as u32, 16)
    }

    fn append_bool(&mut self, value: bool) {
        self.append_int(value as u32, 1);
    }

    fn append_int<T: Into<u32>>(&mut self, value: T, num_bits: usize) {
        let int_value: u32 = value.into();
        let mut remainder: u32 = int_value;
        for _index in 0..num_bits {
            self.push(remainder % 2 == 1);
            remainder /= 2;
        }
        assert_eq!(
            remainder, 0,
            "Tried to write value bigger than {} bits: {}",
            num_bits, int_value
        );
    }

    fn append_optional_int<T: Into<u32>>(&mut self, optional: Option<T>, num_bits: usize) {
        match optional {
            Some(value) => {
                self.append_bool(true);
                self.append_int(value, num_bits);
            }
            None => {
                self.append_bool(false);
            }
        }
    }

    fn append_bits(&mut self, bitvec: &MyBitVec) {
        for bit in bitvec {
            self.append_bool(*bit);
        }
    }

    fn append_optional_bits(&mut self, optional: &Option<MyBitVec>) {
        match optional {
            Some(bits) => {
                self.append_bool(true);
                self.append_bits(bits);
            }
            None => {
                self.append_bool(false);
            }
        }
    }

    fn append_optional_byte_arr<const N: usize>(&mut self, optional: &Option<[u8; N]>) {
        match optional {
            Some(bits) => {
                self.append_bool(true);
                self.extend_from_raw_slice(bits);
            }
            None => {
                self.append_bool(false);
            }
        }
    }

    fn append_byte_arr<const N: usize>(&mut self, array: &[u8; N]) {
        self.extend_from_raw_slice(array);
    }

    fn append_padding(&mut self) {
        while self.len() % 8 != 0 {
            self.append_bool(false);
        }
    }
}

pub fn bitvec_init(value: bool, bits: usize) -> MyBitVec {
    let mut result = MyBitVec::new();
    for _ in 0..bits {
        result.push(value);
    }
    result
}

pub trait Bitsy: BitWritable {
    fn parse<R: BitReader>(reader: &mut R) -> Self;
}

pub trait VersionedBitsy: BitWritable {
    fn parse<R: BitReader>(reader: &mut R, version: u32) -> Self;
}

pub trait BitWritable {
    fn append_to<W: BitWriter>(&self, writer: &mut W);
}

#[derive(Default)]
pub struct Bits<const N: usize> {
    pub bits: MyBitVec,
}

impl<const N: usize> Bitsy for Bits<N> {
    fn parse<R: BitReader>(reader: &mut R) -> Self {
        let bits = reader.read_bits(N);
        Self { bits }
    }
}

impl<const N: usize> BitWritable for Bits<N> {
    fn append_to<W: BitWriter>(&self, writer: &mut W) {
        writer.append_bits(&self.bits);
    }
}

impl<const N: usize> Bitsy for [u8; N] {
    fn parse<R: BitReader>(reader: &mut R) -> Self {
        reader.read_byte_arr()
    }
}

impl<const N: usize> BitWritable for [u8; N] {
    fn append_to<W: BitWriter>(&self, writer: &mut W) {
        writer.append_byte_arr(self);
    }
}

macro_rules! integer_bitsy_impl {
    ($type:ty) => {
        impl Bitsy for $type {
            fn parse<R: BitReader>(reader: &mut R) -> Self {
                reader.read_int(std::mem::size_of::<$type>() * 8)
            }
        }

        impl BitWritable for $type {
            fn append_to<W: BitWriter>(&self, writer: &mut W) {
                writer.append_int(*self, std::mem::size_of::<$type>() * 8);
            }
        }
    };
}

integer_bitsy_impl!(u8);
integer_bitsy_impl!(u16);
integer_bitsy_impl!(u32);
