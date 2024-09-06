use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use crate::bitsy::{result::BitsyResult, BitReader, BitWriter, Bitsy};

#[derive(PartialEq, Eq)]
pub struct BitsyOption<T: Bitsy> {
    value: Option<T>,
}

impl<T: Bitsy> Debug for BitsyOption<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = &self.value {
            write!(f, "BO<{:?}>", value)
        } else {
            write!(f, "BO<None>")
        }
    }
}

impl<T: Bitsy> Deref for BitsyOption<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Bitsy> DerefMut for BitsyOption<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

impl<T: Bitsy> BitsyOption<T> {
    pub fn none() -> Self {
        Self { value: None }
    }

    pub fn some(value: T) -> Self {
        Self { value: Some(value) }
    }
}

impl<T: Bitsy> Bitsy for BitsyOption<T> {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
        let is_present: bool = reader.read()?;
        Ok(if is_present {
            Self::some(reader.read()?)
        } else {
            Self::none()
        })
    }

    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        if let Some(value) = &self.value {
            writer.write(&true)?;
            writer.write(value)?;
        } else {
            writer.write(&false)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::bitsy::{bits_from_str, testutils::assert_reads_to, BitVecReader, BitVecWriter};

    use super::*;

    #[test]
    fn it_reads_none() {
        let mut reader = BitVecReader::dbless(bits_from_str("000101101").unwrap());

        let option: BitsyOption<u8> = reader.read().unwrap();

        assert_eq!(option.value, None);
        assert_eq!(reader.index(), 1);
    }

    #[test]
    fn it_reads_some() {
        let mut reader = BitVecReader::dbless(bits_from_str("100101101").unwrap());

        let option: BitsyOption<u8> = reader.read().unwrap();

        assert_eq!(option.value, Some(0b10110100));
        assert_eq!(reader.index(), 9);
    }

    #[test]
    fn it_writes_none() {
        let mut writer = BitVecWriter::new(0);

        writer.write(&BitsyOption::<u8>::none()).unwrap();

        assert_eq!(bits_from_str("0").unwrap(), writer.into_bits());
    }

    #[test]
    fn it_writes_some() {
        let mut writer = BitVecWriter::new(0);

        writer.write(&BitsyOption::some(0b00101101u8)).unwrap();

        assert_eq!(bits_from_str("110110100").unwrap(), writer.into_bits());
    }

    #[test]
    fn random_roundtrips() {
        for _ in 0..100 {
            let option: BitsyOption<u8> = BitsyOption::some(rand::random());
            let mut writer = BitVecWriter::new(0);
            writer.write(&option).unwrap();
            assert_reads_to(writer.into_bits(), option);
        }
    }
}
