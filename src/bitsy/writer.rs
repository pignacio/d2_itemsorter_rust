use super::{
    error::{BitsyError, BitsyErrorKind},
    result::BitsyResult,
    BitWriter, MyBitVec,
};

pub struct BitVecWriter {
    bits: MyBitVec,
    version: u32,
}

impl BitVecWriter {
    pub fn new(version: u32) -> Self {
        Self {
            bits: MyBitVec::new(),
            version,
        }
    }

    pub fn into_bits(self) -> MyBitVec {
        self.bits
    }
}

impl BitWriter for BitVecWriter {
    fn version(&self) -> Option<u32> {
        Some(self.version)
    }

    fn write_int<T: Into<u32>>(&mut self, value: T, bit_count: usize) -> BitsyResult<()> {
        let int_value: u32 = value.into();
        let mut remainder: u32 = int_value;
        for _index in 0..bit_count {
            self.bits.push(remainder % 2 == 1);
            remainder /= 2;
        }
        if remainder != 0 {
            return Err(BitsyError::new(
                BitsyErrorKind::InvalidData(format!(
                    "Tried to write value bigger than {bit_count} bits: {int_value}",
                )),
                self.bits.len() - bit_count,
            ));
        }
        Ok(())
    }

    fn write_bits(&mut self, value: &MyBitVec) -> BitsyResult<()> {
        self.bits.extend_from_bitslice(value);
        Ok(())
    }

    fn write_padding(&mut self) -> BitsyResult<()> {
        while self.bits.len() % 8 != 0 {
            self.bits.push(false);
        }
        Ok(())
    }
}
