use std::convert::TryFrom;

use bitvec::prelude::*;
use bitvec::view::BitViewSized;

pub type MyBitOrder = Lsb0;
pub type MyBitVec = BitVec<MyBitOrder, u8>;

pub struct BitReader {
    bytes: Vec<u8>,
    bits: MyBitVec,
    index: usize,
}

impl BitReader {
    pub fn new(bytes: Vec<u8>) -> BitReader {
        return BitReader {
            bytes: bytes.to_vec(),
            bits: BitVec::from_vec(bytes),
            index: 0,
        };
    }

    pub fn get(&self, index: usize) -> bool {
        return self.bits[index];
    }

    pub fn index(&self) -> usize {
        return self.index;
    }

    pub fn read_byte_arr<const N: usize>(&mut self) -> [u8; N] {
        let mut result = [0; N];

        for index in 0..N {
            result[index] = self.read_int(8);
        }
        return result;
    }

    pub fn read_optional_byte_arr<const N: usize>(&mut self) -> Option<[u8; N]> {
        let is_present = self.read_bool();
        return if is_present {
            Some(self.read_byte_arr())
        } else {
            None
        };
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

    pub fn read_bits(&mut self, size: usize) -> MyBitVec {
        let mut bitvec = BitVec::new();
        for bit in self.bits[self.index..].iter().take(size) {
            bitvec.push(*bit);
        }
        self.index += size;
        return bitvec;
    }

    pub fn read_optional_bits(&mut self, num_bits: usize) -> Option<MyBitVec> {
        let is_present = self.read_bool();
        return if is_present {
            Some(self.read_bits(num_bits))
        } else {
            None
        };
    }

    pub fn read_bool(&mut self) -> bool {
        return self.read_int::<u32>(1) != 0;
    }

    pub fn read_int<T: TryFrom<u32>>(&mut self, num_bits: usize) -> T {
        assert!(num_bits <= 32, "Support for ints > 32 bits missing");
        let mut res: u32 = 0;
        let mut multiplier = 1;

        for index in 0..num_bits {
            res += self.bits[self.index + index] as u32 * multiplier;
            if index < 31 {
                multiplier *= 2;
            }
        }

        self.index += num_bits;
        return T::try_from(res).unwrap_or_else(|_| panic!("Int did not fit"));
    }

    pub fn read_optional_int<T: TryFrom<u32>>(&mut self, num_bits: usize) -> Option<T> {
        let is_present = self.read_bool();
        return if is_present {
            Some(self.read_int(num_bits))
        } else {
            None
        };
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
        for index in bit_start_index..(8 * self.bytes.len()) {
            let bit = self.get(index);
            if bit == sentinel[current_match_size] {
                // println!("Partial match! index:{}, current match:{}, next byte:{}", index, current_match_size, self.bytes[index + 1]);
                current_match_size += 1;
                if current_match_size == sentinel.len() {
                    return Some(index + 1);
                }
            } else {
                current_match_size = 0;
            }
        }
        None
    }

    pub fn read_until(&mut self, sentinel: &[u8]) -> MyBitVec {
        let start = self.index;

        return match self.find_match_index(sentinel) {
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
        };
    }

    pub fn read_until_bits(&mut self, sentinel: &MyBitVec) -> MyBitVec {
        let start = self.index;
        return match self.find_bits_match_index(sentinel) {
            Some(index) => {
                self.index = index;
                self.bits[start..self.index].to_bitvec()
            }
            _ => {
                self.bits.len();
                self.bits[start..].to_bitvec()
            }
        };
    }

    pub fn tail(&mut self) -> MyBitVec {
        return self.bits[self.index..].to_bitvec();
    }
}

pub trait BitWriter {
    fn append_u32(&mut self, value: u32);
    fn append_u16(&mut self, value: u16);
    fn append_bool(&mut self, value: bool);
    fn append_int<T: Into<u32>>(&mut self, value: T, num_bits: usize);
    fn append_optional_int<T: Into<u32>>(&mut self, optional: Option<T>, num_bits: usize);
    fn append_bitarr<O: BitOrder, V: BitViewSized>(&mut self, array: &BitArray<O, V>);
    fn append_bits(&mut self, bitvec: &MyBitVec);
    fn append_optional_bits(&mut self, optional: &Option<MyBitVec>);
    fn append_optional_byte_arr<const N: usize>(&mut self, optional: &Option<[u8; N]>);
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

    fn append_bitarr<O: BitOrder, V: BitViewSized>(&mut self, array: &BitArray<O, V>) {
        println!("Array length: {}", array.len());
        panic!();
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
                self.append_bits(&bits);
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
}

pub fn bitvec_init(value: bool, bits: usize) -> MyBitVec {
    let mut result = MyBitVec::new();
    for _ in 0..bits {
        result.push(value);
    }
    return result;
}
