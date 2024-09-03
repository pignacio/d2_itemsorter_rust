use std::fmt::Debug;

use crate::bitsy::{
    error::BitsyErrorKind, result::BitsyResult, BitReader, BitSized, BitWriter, Bitsy, MyBitSlice,
    MyBitVec,
};

pub struct Bits<const N: usize> {
    bits: MyBitVec,
}

impl<const N: usize> Bits<N> {
    pub fn new(bits: MyBitVec) -> BitsyResult<Self> {
        if bits.len() != N {
            return Err(BitsyErrorKind::InvalidData(format!(
                "Wrong number of bits. Expected {N} and got {}",
                bits.len()
            ))
            .at_bit(0));
        }
        Ok(Self { bits })
    }

    pub fn as_bitslice(&self) -> &MyBitSlice {
        self.bits.as_bitslice()
    }

    pub fn to_bitvec(&self) -> MyBitVec {
        self.bits.clone()
    }
}

impl<const N: usize> Debug for Bits<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.bits.to_string().replace(", ", ""))
    }
}

impl<const N: usize> Bitsy for Bits<N> {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
        Ok(Self {
            bits: reader.read_bits(N)?,
        })
    }

    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        writer.write_bits(&self.bits)
    }
}

impl<const N: usize> BitSized for Bits<N> {
    fn bit_size(&self) -> usize {
        N
    }
}

#[cfg(test)]
mod tests {
    use crate::bitsy::{
        bits_from_str,
        testutils::{compare_bitslices, random_bits},
        BitVecReader, BitVecWriter,
    };

    use super::*;
    #[test]
    fn it_reads() {
        let mut reader = BitVecReader::new(bits_from_str("010011").unwrap());
        let mut bits: Bits<3>;

        bits = reader.read().unwrap();
        compare_bitslices(&bits_from_str("010").unwrap(), bits.as_bitslice()).unwrap();
        assert_eq!(reader.index(), 3);
        bits = reader.read().unwrap();
        compare_bitslices(&bits_from_str("011").unwrap(), bits.as_bitslice()).unwrap();
        assert_eq!(reader.index(), 6);
    }

    #[test]
    fn it_writes() {
        let mut writer = BitVecWriter::new(0);
        let bits = Bits::<3>::new(bits_from_str("010").unwrap()).unwrap();
        bits.write_to(&mut writer).unwrap();
        compare_bitslices(
            &bits_from_str("010").unwrap(),
            writer.into_bits().as_bitslice(),
        )
        .unwrap();

        writer = BitVecWriter::new(0);
        let bits = Bits::<3>::new(bits_from_str("011").unwrap()).unwrap();
        bits.write_to(&mut writer).unwrap();
        compare_bitslices(
            &bits_from_str("011").unwrap(),
            writer.into_bits().as_bitslice(),
        )
        .unwrap();
    }

    #[test]
    fn roundtrip_random_bits() {
        for _index in 0..1000 {
            let bitvec = random_bits(16);
            let bits = Bits::<16>::new(bitvec.clone()).unwrap();

            compare_bitslices(&bitvec, bits.as_bitslice()).unwrap();
        }
    }
}
