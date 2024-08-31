use std::{
    convert::TryFrom,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

use super::{error::BitsyErrorExt, BitReader, BitWriter, Bitsy, BitsyResult, MyBitVec};

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

pub struct Bits<const N: usize> {
    bits: MyBitVec,
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

pub struct BitsyInt<T: TryFrom<u32> + Into<u32> + Copy, const N: usize> {
    value: T,
}

impl<T: TryFrom<u32> + Into<u32> + Copy + Debug, const N: usize> Debug for BitsyInt<T, N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BI<{:?}>", self.value)
    }
}

impl<T: TryFrom<u32> + Into<u32> + Copy, const N: usize> BitsyInt<T, N> {
    pub fn value(&self) -> T {
        self.value
    }
}

//impl<T: TryFrom<u32> + Into<u32> + Copy, const N: usize> Deref for BitsyInt<T, N> {
//    type Target = T;
//
//    fn deref(&self) -> &Self::Target {
//        &self.value
//    }
//}

impl<T: TryFrom<u32> + Into<u32> + Copy, const N: usize> Bitsy for BitsyInt<T, N> {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
        Ok(Self {
            value: reader.read_int::<T>(N)?,
        })
    }

    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        writer.write_int(self.value, N)
    }
}

pub struct BitsyOption<T: Bitsy> {
    value: Option<T>,
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
