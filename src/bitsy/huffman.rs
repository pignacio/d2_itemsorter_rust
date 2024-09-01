use std::fmt::Debug;

use crate::bitsy::error::{BitsyError, BitsyErrorKind};

use super::{error::BitsyErrorExt, result::BitsyResult, Bitsy};

lazy_static::lazy_static! {
    pub static ref HUFFMAN_DECODE_MAP: std::collections::HashMap<String, char> =
        [
            ("111101000", '\0'),
            ("01", ' '),
            ("11011111", '0'),
            ("0011111", '1'),
            ("001100", '2'),
            ("1011011", '3'),
            ("01011111", '4'),
            ("01101000", '5'),
            ("1111011", '6'),
            ("11110", '7'),
            ("001000", '8'),
            ("01110", '9'),
            ("01111", 'a'),
            ("1010", 'b'),
            ("00010", 'c'),
            ("100011", 'd'),
            ("000011", 'e'),
            ("110010", 'f'),
            ("01011", 'g'),
            ("11000", 'h'),
            ("0111111", 'i'),
            ("011101000", 'j'),
            ("010010", 'k'),
            ("10111", 'l'),
            ("10110", 'm'),
            ("101100", 'n'),
            ("1111111", 'o'),
            ("10011", 'p'),
            ("10011011", 'q'),
            ("00111", 'r'),
            ("0100", 's'),
            ("00110", 't'),
            ("10000", 'u'),
            ("0111011", 'v'),
            ("00000", 'w'),
            ("11100", 'x'),
            ("0101000", 'y'),
            ("00011011", 'z')
        ].iter()
            .map(|(k, v)| (k.chars().rev().collect::<String>(), *v))
            .collect();
}

const MAX_HUFFMAN_SIZE: usize = 9;

#[derive(Clone, Copy, Default)]
pub struct HuffmanChar {
    pub char: char,
}

impl Bitsy for HuffmanChar {
    fn parse<R: super::BitReader>(reader: &mut R) -> BitsyResult<Self> {
        let mut key = String::new();
        loop {
            if key.len() >= MAX_HUFFMAN_SIZE {
                return Err(BitsyError::new(
                    BitsyErrorKind::InvalidData(format!("Could not match huffman code '{key}'")),
                    reader.index() - key.len(),
                ));
            }
            key.push(if reader.read::<bool>()? { '1' } else { '0' });
            if let Some(c) = HUFFMAN_DECODE_MAP.get(&key) {
                return Ok(HuffmanChar { char: *c });
            }
        }
    }

    fn write_to<W: super::BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        todo!()
    }
}

pub struct HuffmanChars<const N: usize> {
    chars: [HuffmanChar; N],
}

impl<const N: usize> HuffmanChars<N> {
    pub fn as_string(&self) -> String {
        self.chars.iter().map(|c| c.char).collect::<String>()
    }
}

impl<const N: usize> Bitsy for HuffmanChars<N> {
    fn parse<R: super::BitReader>(reader: &mut R) -> BitsyResult<Self> {
        let mut chars = [HuffmanChar::default(); N];
        for (i, char) in chars.iter_mut().enumerate() {
            *char = reader.read().prepend_index(i)?;
        }
        Ok(HuffmanChars { chars })
    }

    fn write_to<W: super::BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        for (i, c) in self.chars.iter().enumerate() {
            c.write_to(writer).prepend_index(i)?;
        }
        Ok(())
    }
}

impl<const N: usize> Debug for HuffmanChars<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HC<{}>", self.as_string())
    }
}

//{"111101000", '\0'}, {       "01", ' '}, {"11011111", '0'}, { "0011111", '1'},
//{   "001100",  '2'}, {  "1011011", '3'}, {"01011111", '4'}, {"01101000", '5'},
//{  "1111011",  '6'}, {    "11110", '7'}, {  "001000", '8'}, {   "01110", '9'},
//{    "01111",  'a'}, {     "1010", 'b'}, {   "00010", 'c'}, {  "100011", 'd'},
//{   "000011",  'e'}, {   "110010", 'f'}, {   "01011", 'g'}, {   "11000", 'h'},
//{  "0111111",  'i'}, {"011101000", 'j'}, {  "010010", 'k'}, {   "10111", 'l'},
//{    "10110",  'm'}, {   "101100", 'n'}, { "1111111", 'o'}, {   "10011", 'p'},
//{ "10011011",  'q'}, {    "00111", 'r'}, {    "0100", 's'}, {   "00110", 't'},
//{    "10000",  'u'}, {  "0111011", 'v'}, {   "00000", 'w'}, {   "11100", 'x'},
//{  "0101000",  'y'}, { "00011011", 'z'},
//
//("00000", 'w'),
//("000011", 'e'),
//("00010", 'c'),
//("00011011", 'z')
//("001000", '8'),
//("001100", '2'),
//("00110", 't'),
//("0011111", '1'),
//("00111", 'r'),
//("010010", 'k'),
//("0100", 's'),
//("0101000", 'y'),
//("01011111", '4'),
//("01011", 'g'),
//("01101000", '5'),
//("011101000", 'j'),
//("0111011", 'v'),
//("01110", '9'),
//("0111111", 'i'),
//("01111", 'a'),
//("01", ' '),
//("10000", 'u'),
//("100011", 'd'),
//("10011011", 'q'),
//("10011", 'p'),
//("1010", 'b'),
//("101100", 'n'),
//("1011011", '3'),
//("10110", 'm'),
//("10111", 'l'),
//("11000", 'h'),
//("110010", 'f'),
//("11011111", '0'),
//("11100", 'x'),
//("111101000", '\0'),
//("1111011", '6'),
//("11110", '7'),
//("1111111", 'o'),
//

#[cfg(test)]
mod tests {
    use crate::bitsy::{BitReader, BitVecReader, MyBitVec};

    use super::*;
    #[test]
    fn it_works() {
        let bitvec = MyBitVec::from_vec(vec![0xB1, 0xF2, 0x18, 0x3A]);
        let mut reader = BitVecReader::new(bitvec);

        reader.read_bits(3).unwrap();
        let chars = reader.read::<HuffmanChars<4>>().unwrap();
        println!("Parsed: {chars:?}");
    }
}
