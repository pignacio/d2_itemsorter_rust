use std::{convert::TryInto, fmt::Display};

use crate::bitsy::{
    error::BitsyErrorKind, result::BitsyResult, BitReader, BitSized, BitWriter, Bitsy,
};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct BitsyChars<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> Display for BitsyChars<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.bytes
            .iter()
            .map(|byte| *byte as char)
            .try_for_each(|c| c.fmt(f))
    }
}

impl<const N: usize> BitsyChars<N> {
    pub fn new<S: AsRef<str>>(string: S) -> BitsyResult<Self> {
        let chars = string.as_ref().chars().collect::<Vec<char>>();
        if chars.len() != N {
            return Err(BitsyErrorKind::InvalidData(format!(
                "Expected {} characters, got {}",
                N,
                chars.len()
            ))
            .at_bit(0));
        }
        let mut bytes = [0u8; N];
        for index in 0..N {
            let byte: u8 = chars[index].try_into().map_err(|err| {
                BitsyErrorKind::InvalidData(format!(
                    "Could not map char '{:?}' to a byte: {}",
                    chars[index], err
                ))
                .at_bit(0)
            })?;

            bytes[index] = byte;
        }
        Ok(Self { bytes })
    }
}

impl<const N: usize> Bitsy for BitsyChars<N> {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
        reader.read().map(|bytes| Self { bytes })
    }

    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        writer.write(&self.bytes)
    }
}

impl<const N: usize> std::fmt::Debug for BitsyChars<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self
            .bytes
            .iter()
            .map(|byte| *byte as char)
            .collect::<String>();

        write!(f, "BC<{:?}>", string)
    }
}

impl<const N: usize> BitSized for BitsyChars<N> {
    fn bit_size(&self) -> usize {
        N * 8
    }
}

#[cfg(test)]
mod tests {
    use crate::bitsy::{
        compare_bitslices, testutils::assert_reads_to, BitVecReader, BitVecWriter, MyBitVec,
    };

    use super::*;

    #[test]
    fn it_reads() {
        let bits = MyBitVec::from_vec("hello".chars().map(|c| c as u8).collect());
        let mut reader = BitVecReader::dbless(bits);

        let chars: BitsyChars<4> = reader.read().unwrap();

        assert_eq!(BitsyChars::new("hell").unwrap(), chars);
        assert_eq!(reader.index(), 4 * 8);
    }

    #[test]
    fn it_writes() {
        let chars = BitsyChars::<5>::new("hello").unwrap();
        let mut writer = BitVecWriter::new(0);
        writer.write(&chars).unwrap();

        compare_bitslices(
            &MyBitVec::from_vec("hello".chars().map(|c| c as u8).collect()),
            &writer.into_bits(),
        )
        .unwrap();
    }

    const VALID_CHARS: [char; 62] = [
        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r',
        's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J',
        'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1',
        '2', '3', '4', '5', '6', '7', '8', '9',
    ];
    fn random_string(length: usize) -> String {
        let mut string = String::new();
        for _ in 0..length {
            string.push(VALID_CHARS[rand::random::<usize>() % VALID_CHARS.len()]);
        }
        string
    }

    #[test]
    fn random_rountrips() {
        for _ in 0..100 {
            let string = random_string(16);
            let original = BitsyChars::<16>::new(string).unwrap();

            let mut writer = BitVecWriter::new(0);
            writer.write(&original).unwrap();
            assert_reads_to(writer.into_bits(), original);
        }
    }
}
