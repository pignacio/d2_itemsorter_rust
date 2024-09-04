use super::{
    error::{BitsyErrorExt, BitsyErrorKind},
    BitReader, BitSized, BitWriter, Bitsy, BitsyResult, MyBitVec,
};

macro_rules! integer_bitsy_impl {
    ($ty:ty) => {
        impl Bitsy for $ty {
            fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
                reader.read_int::<$ty>(std::mem::size_of::<$ty>() * 8)
            }

            fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
                writer.write_int(*self, std::mem::size_of::<$ty>() * 8)
            }
        }

        impl BitSized for $ty {
            fn bit_size(&self) -> usize {
                std::mem::size_of::<$ty>() * 8
            }
        }
    };
}

integer_bitsy_impl!(u8);
integer_bitsy_impl!(u16);
integer_bitsy_impl!(u32);

impl Bitsy for bool {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
        reader.read_int::<u8>(1).map(|x| x != 0)
    }

    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        writer.write_int(*self as u8, 1)
    }
}

impl BitSized for bool {
    fn bit_size(&self) -> usize {
        1
    }
}

impl<const N: usize> Bitsy for [u8; N] {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
        let mut buf = [0u8; N];
        for (index, value) in buf.iter_mut().enumerate() {
            *value = reader.read().prepend_index(index)?;
        }
        Ok(buf)
    }

    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        for (index, value) in self.iter().enumerate() {
            writer.write(value).prepend_index(index)?;
        }
        Ok(())
    }
}

impl<const N: usize, T: BitSized> BitSized for [T; N] {
    fn bit_size(&self) -> usize {
        self.iter().map(|x| x.bit_size()).sum()
    }
}

impl<T: Bitsy> Bitsy for Option<T> {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
        reader.read().map(|v| Some(v))
    }

    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        if let Some(value) = self {
            writer.write(value)?;
        }
        Ok(())
    }
}

impl BitSized for MyBitVec {
    fn bit_size(&self) -> usize {
        self.len()
    }
}

impl<T: Bitsy> Bitsy for Vec<T> {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
        Err(
            BitsyErrorKind::InvalidAction("Cannot read Vec<T: Bitsy> directly!".to_string())
                .at_bit(reader.index()),
        )
    }

    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        for (index, value) in self.iter().enumerate() {
            writer.write(value).prepend_index(index)?;
        }
        Ok(())
    }
}
