use super::{
    context::ContextMap,
    error::{BitsyError, BitsyErrorKind},
    result::BitsyResult,
    BitWriter, MyBitVec,
};

pub struct BitVecWriter {
    bits: MyBitVec,
    context: ContextMap,
}

impl BitVecWriter {
    pub fn new(version: u32) -> Self {
        let mut writer = Self {
            bits: MyBitVec::new(),
            context: ContextMap::new(),
        };

        writer.set_context(&super::context::VERSION, version);
        writer
    }

    pub fn into_bits(self) -> MyBitVec {
        self.bits
    }
}

impl BitWriter for BitVecWriter {
    fn index(&self) -> usize {
        self.bits.len()
    }

    fn queue_context_reset(&self) -> super::context::ContextResetGuard {
        self.context.context_reset()
    }

    fn set_context<T: super::context::ContextValue>(
        &mut self,
        key: &super::context::ContextKey<T>,
        value: T,
    ) {
        self.context.set_context(key, value);
    }

    fn get_context<T: super::context::ContextValue>(
        &self,
        key: &super::context::ContextKey<T>,
    ) -> BitsyResult<T> {
        self.context.get_context(key).ok_or_else(|| {
            BitsyError::new(
                BitsyErrorKind::MissingContext(key.as_str().to_string()),
                self.bits.len(),
            )
        })
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
