use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::bitsy::{result::BitsyResult, BitReader, BitWriter, Bitsy};

pub struct BitsyBytes<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> BitsyBytes<N> {
    pub fn new(bytes: [u8; N]) -> Self {
        Self { bytes }
    }
}

impl<const N: usize> Deref for BitsyBytes<N> {
    type Target = [u8; N];

    fn deref(&self) -> &Self::Target {
        &self.bytes
    }
}

impl<const N: usize> DerefMut for BitsyBytes<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bytes
    }
}

impl<const N: usize> Bitsy for BitsyBytes<N> {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
        reader.read().map(|bytes| Self { bytes })
    }

    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        writer.write(&self.bytes)
    }
}

const MAX_DEBUG_BYTES: usize = 12;

fn format_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .copied()
        .map(format_byte)
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_byte(byte: u8) -> String {
    format!("{byte} (0x{byte:02X})")
}

impl<const N: usize> Debug for BitsyBytes<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.bytes.len() > MAX_DEBUG_BYTES {
            let start_bytes = format_bytes(&self.bytes[..MAX_DEBUG_BYTES / 2]);
            let end_bytes = format_bytes(&self.bytes[self.bytes.len() - MAX_DEBUG_BYTES / 2..]);
            write!(
                f,
                "[{start_bytes} ... {} bytes hidden ... {end_bytes}]",
                self.bytes.len() - MAX_DEBUG_BYTES,
            )
        } else {
            write!(f, "[{}]", format_bytes(&self.bytes))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bitsy::{compare_bitslices, testutils::bits, BitVecReader, BitVecWriter, MyBitVec};

    use super::*;

    #[test]
    fn it_reads_aligned_bytes() {
        let original = [rand::random::<u8>(); 16].to_vec();
        let mut reader = BitVecReader::dbless(MyBitVec::from_vec(original.clone()));

        let bytes: BitsyBytes<16> = reader.read().unwrap();

        assert_eq!(original, bytes.deref());
        assert_eq!(reader.index(), 16 * 8);
    }

    #[test]
    fn it_reads_unaligned_bytes() {
        let original = [rand::random::<u8>(); 16].to_vec();
        let mut bit_vec = bits("010");
        bit_vec.extend_from_raw_slice(&original);
        let mut reader = BitVecReader::dbless(bit_vec);

        reader.read_bits(3).unwrap();

        let bytes: BitsyBytes<16> = reader.read().unwrap();

        assert_eq!(original, bytes.deref());
        assert_eq!(reader.index(), 16 * 8 + 3);
    }

    #[test]
    fn it_writes() {
        let first = BitsyBytes::<2>::new([0b00101101, 0b10110100]);
        let second = BitsyBytes::<1>::new([0b01011011]);

        let mut writer = BitVecWriter::new(0);
        writer.write(&first).unwrap();
        writer.write_bits(&bits("010")).unwrap();
        writer.write(&second).unwrap();

        let expected = bits("10110100 00101101 010 11011010");
        compare_bitslices(&expected, &writer.into_bits()).unwrap();
    }
}
