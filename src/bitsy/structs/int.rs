use std::{
    convert::TryFrom,
    fmt::{Debug, Display},
};

use crate::bitsy::{
    error::BitsyErrorKind, result::BitsyResult, BitReader, BitSized, BitWriter, Bitsy,
};

pub trait BitsyIntTarget: TryFrom<u32> + Into<u32> + Copy + Debug + Ord {}
impl<T: TryFrom<u32> + Into<u32> + Copy + Debug + Ord> BitsyIntTarget for T {}

#[derive(Clone, Copy)]
pub struct BitsyInt<T: BitsyIntTarget, const N: usize> {
    value: T,
}

impl<T: BitsyIntTarget, const N: usize> Debug for BitsyInt<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BI<{:?}>", self.value)
    }
}

impl<T: BitsyIntTarget, const N: usize> Display for BitsyInt<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl<T: BitsyIntTarget, const N: usize> BitsyInt<T, N> {
    pub fn new(value: T) -> BitsyResult<Self> {
        if value.into() >= (2u32 << N) {
            return Err(BitsyErrorKind::InvalidData(format!(
                "Value {value:?} is too large for {N} bits",
            ))
            .at_bit(0));
        }

        Ok(Self { value })
    }

    pub fn value(&self) -> T {
        self.value
    }
}

impl<T: BitsyIntTarget, const N: usize> Bitsy for BitsyInt<T, N> {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
        Ok(Self {
            value: reader.read_int::<T>(N)?,
        })
    }

    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        writer.write_int(self.value, N)
    }
}

impl<T: BitsyIntTarget, const N: usize> BitSized for BitsyInt<T, N> {
    fn bit_size(&self) -> usize {
        N
    }
}

#[cfg(test)]
mod tests {
    use crate::bitsy::{
        bits_from_str,
        testutils::{compare_bitslices, random_bits},
        BitVecReader, BitVecWriter, MyBitVec,
    };

    use super::*;

    #[test]
    fn it_reads() {
        let bits = bits_from_str("00101110").unwrap();
        let mut reader = BitVecReader::new(bits);

        let int: BitsyInt<u8, 6> = reader.read().unwrap();

        assert_eq!(52, int.value());
        assert_eq!(reader.index(), 6);
    }

    #[test]
    fn it_writes() {
        let int = BitsyInt::<u8, 6>::new(52).unwrap();
        let mut writer = BitVecWriter::new(0);
        writer.write(&int).unwrap();

        compare_bitslices(&bits_from_str("001011").unwrap(), &writer.into_bits()).unwrap();
    }

    #[test]
    fn random_rountrips() {
        for _ in 0..100 {
            let bits = random_bits(6);
            let mut reader = BitVecReader::new(bits.clone());
            let int: BitsyInt<u8, 6> = reader.read().unwrap();
            let mut writer = BitVecWriter::new(0);
            writer.write(&int).unwrap();
            compare_bitslices(&bits, &writer.into_bits()).unwrap();
        }
    }
}
