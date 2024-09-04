use serde::{Deserialize, Serialize};

use crate::bitsy::{
    context,
    error::{BitsyError, BitsyErrorKind},
    macros::{bitsy_read, bitsy_write},
    result::BitsyResult,
    structs::{Bits, BitsyInt, BitsyOption},
    BitReader, BitSized, BitWriter, Bitsy, MyBitVec, OldBitReader, OldBitWriter,
};
use std::fmt::{Display, Formatter};

pub const SET_QUALITY_ID: u8 = 5;

pub trait Quality: Display {
    fn quality_id(&self) -> u8;
    fn write_quality_bytes(&self, bitvec: &mut MyBitVec);
    fn read_quality_bytes(id: u8, bitreader: &mut OldBitReader) -> Box<dyn Quality>
    where
        Self: Sized;
}

pub struct NormalQuality {
    id: u8,
}

impl NormalQuality {
    pub fn default() -> NormalQuality {
        return NormalQuality { id: 15 };
    }
}

impl Quality for NormalQuality {
    fn quality_id(&self) -> u8 {
        return self.id;
    }

    fn write_quality_bytes(&self, _bitvec: &mut MyBitVec) {
        // No extra bits
    }

    fn read_quality_bytes(id: u8, _bitreader: &mut OldBitReader) -> Box<dyn Quality> {
        return Box::new(NormalQuality { id });
    }
}

impl Display for NormalQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "normal");
    }
}

pub struct LowQuality {
    low_quality_type: u8,
}

impl Quality for LowQuality {
    fn quality_id(&self) -> u8 {
        return 1;
    }

    fn write_quality_bytes(&self, bitvec: &mut MyBitVec) {
        bitvec.append_int(self.low_quality_type as u32, 3);
    }

    fn read_quality_bytes(id: u8, bitreader: &mut OldBitReader) -> Box<dyn Quality> {
        assert_eq!(id, 1, "Low quality should have id = 1");
        return Box::new(LowQuality {
            low_quality_type: bitreader.read_int(3),
        });
    }
}

impl Display for LowQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "low({})", self.low_quality_type);
    }
}

pub struct HighQuality {
    high_quality_type: u8,
}

impl Quality for HighQuality {
    fn quality_id(&self) -> u8 {
        return 3;
    }

    fn write_quality_bytes(&self, bitvec: &mut MyBitVec) {
        bitvec.append_int(self.high_quality_type as u32, 3);
    }

    fn read_quality_bytes(id: u8, bitreader: &mut OldBitReader) -> Box<dyn Quality> {
        assert_eq!(id, 3, "High quality should have id = 3");
        return Box::new(HighQuality {
            high_quality_type: bitreader.read_int(3),
        });
    }
}

impl Display for HighQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "low({})", self.high_quality_type);
    }
}

pub struct MagicQuality {
    prefix: u16,
    suffix: u16,
}

impl Quality for MagicQuality {
    fn quality_id(&self) -> u8 {
        return 4;
    }

    fn write_quality_bytes(&self, bitvec: &mut MyBitVec) {
        bitvec.append_int(self.prefix as u32, 11);
        bitvec.append_int(self.suffix as u32, 11);
    }

    fn read_quality_bytes(id: u8, bitreader: &mut OldBitReader) -> Box<dyn Quality> {
        assert_eq!(id, 4, "Magic quality should have id = 4");
        return Box::new(MagicQuality {
            prefix: bitreader.read_int(11),
            suffix: bitreader.read_int(11),
        });
    }
}

impl Display for MagicQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "magic(pre:{}, suf:{})", self.prefix, self.suffix);
    }
}

pub struct SetQuality {
    set_id: u16,
}

impl Quality for SetQuality {
    fn quality_id(&self) -> u8 {
        return SET_QUALITY_ID;
    }

    fn write_quality_bytes(&self, bitvec: &mut MyBitVec) {
        bitvec.append_int(self.set_id as u32, 12);
    }

    fn read_quality_bytes(id: u8, bitreader: &mut OldBitReader) -> Box<dyn Quality> {
        assert_eq!(id, SET_QUALITY_ID, "Set quality should have id = 5");
        return Box::new(SetQuality {
            set_id: bitreader.read_int(12),
        });
    }
}

impl Display for SetQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "set({})", self.set_id);
    }
}

pub struct RareQuality {
    id: u8,
    first_name: u8,
    last_name: u8,
    prefix1: Option<u16>,
    prefix2: Option<u16>,
    prefix3: Option<u16>,
    suffix1: Option<u16>,
    suffix2: Option<u16>,
    suffix3: Option<u16>,
}

// This includes crafted (id 8) along with rare (id 6)
impl Quality for RareQuality {
    fn quality_id(&self) -> u8 {
        return self.id;
    }

    fn write_quality_bytes(&self, bitvec: &mut MyBitVec) {
        bitvec.append_int(self.first_name, 8);
        bitvec.append_int(self.last_name, 8);
        bitvec.append_optional_int(self.prefix1, 11);
        bitvec.append_optional_int(self.suffix1, 11);
        bitvec.append_optional_int(self.prefix2, 11);
        bitvec.append_optional_int(self.suffix2, 11);
        bitvec.append_optional_int(self.prefix3, 11);
        bitvec.append_optional_int(self.suffix3, 11);
    }

    fn read_quality_bytes(id: u8, bitreader: &mut OldBitReader) -> Box<dyn Quality> {
        assert!(
            id == 6 || id == 8,
            "Rare/Crafted quality should have id in [6, 8]"
        );
        return Box::new(RareQuality {
            id,
            first_name: bitreader.read_int(8),
            last_name: bitreader.read_int(8),
            prefix1: bitreader.read_optional_int(11),
            suffix1: bitreader.read_optional_int(11),
            prefix2: bitreader.read_optional_int(11),
            suffix2: bitreader.read_optional_int(11),
            prefix3: bitreader.read_optional_int(11),
            suffix3: bitreader.read_optional_int(11),
        });
    }
}

impl Display for RareQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "{}({} {})",
            if self.id == 6 { "rare" } else { "crafted" },
            self.first_name,
            self.last_name,
        );
    }
}

pub struct UniqueQuality {
    unique_id: u16,
}

impl Quality for UniqueQuality {
    fn quality_id(&self) -> u8 {
        return 7;
    }

    fn write_quality_bytes(&self, bitvec: &mut MyBitVec) {
        bitvec.append_int(self.unique_id, 12);
    }

    fn read_quality_bytes(id: u8, bitreader: &mut OldBitReader) -> Box<dyn Quality> {
        assert_eq!(id, 7, "Unique quality should have id = 7");
        return Box::new(UniqueQuality {
            unique_id: bitreader.read_int(12),
        });
    }
}

impl Display for UniqueQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "unique({})", self.unique_id,);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum QualityId {
    Normal,
    Low,
    High,
    Magic,
    Set,
    Rare,
    Unique,
    Crafted,
}

impl Bitsy for QualityId {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
        let id: BitsyInt<u8, 4> = reader.read()?;
        match id.value() {
            1 => Ok(QualityId::Low),
            2 => Ok(QualityId::Normal),
            3 => Ok(QualityId::High),
            4 => Ok(QualityId::Magic),
            5 => Ok(QualityId::Set),
            6 => Ok(QualityId::Rare),
            7 => Ok(QualityId::Unique),
            8 => Ok(QualityId::Crafted),
            value => Err(
                BitsyErrorKind::InvalidData(format!("Invalid quality id {}", value))
                    .at_bit(reader.index() - id.bit_size()),
            ),
        }
    }

    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        let id: u8 = match self {
            QualityId::Low => 1,
            QualityId::Normal => 2,
            QualityId::High => 3,
            QualityId::Magic => 4,
            QualityId::Set => 5,
            QualityId::Rare => 6,
            QualityId::Unique => 7,
            QualityId::Crafted => 8,
        };
        writer.write_int(id, 4)?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum ItemQuality {
    Low(Bits<4>),
    Normal,
    Superior(Bits<3>),
    Magic {
        prefix: Bits<11>,
        suffix: Bits<11>,
    },
    Set {
        id: Bits<12>,
    },
    Rare {
        first_name: Bits<8>,
        last_name: Bits<8>,
        prefix1: BitsyOption<Bits<11>>,
        suffix1: BitsyOption<Bits<11>>,
        prefix2: BitsyOption<Bits<11>>,
        suffix2: BitsyOption<Bits<11>>,
        prefix3: BitsyOption<Bits<11>>,
        suffix3: BitsyOption<Bits<11>>,
    },
    Unique {
        id: Bits<12>,
    },
    Crafted {
        first_name: Bits<8>,
        last_name: Bits<8>,
        prefix1: BitsyOption<Bits<11>>,
        suffix1: BitsyOption<Bits<11>>,
        prefix2: BitsyOption<Bits<11>>,
        suffix2: BitsyOption<Bits<11>>,
        prefix3: BitsyOption<Bits<11>>,
        suffix3: BitsyOption<Bits<11>>,
    },
}

impl ItemQuality {
    pub fn get_quality_id(&self) -> QualityId {
        match self {
            ItemQuality::Low(_) => QualityId::Low,
            ItemQuality::Normal => QualityId::Normal,
            ItemQuality::Superior(_) => QualityId::High,
            ItemQuality::Magic { .. } => QualityId::Magic,
            ItemQuality::Set { .. } => QualityId::Set,
            ItemQuality::Rare { .. } => QualityId::Rare,
            ItemQuality::Unique { .. } => QualityId::Unique,
            ItemQuality::Crafted { .. } => QualityId::Crafted,
        }
    }
}

impl Bitsy for ItemQuality {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
        let quality_id = reader.get_context(&context::QUALITY_ID)?;
        match quality_id {
            QualityId::Low => Ok(ItemQuality::Low(reader.read()?)),
            QualityId::Normal => Ok(ItemQuality::Normal),
            QualityId::High => Ok(ItemQuality::Superior(reader.read()?)),
            QualityId::Magic => {
                bitsy_read!(reader, prefix, suffix);
                Ok(ItemQuality::Magic { prefix, suffix })
            }
            QualityId::Set => Ok(ItemQuality::Set { id: reader.read()? }),
            QualityId::Rare => {
                bitsy_read!(
                    reader, first_name, last_name, prefix1, suffix1, prefix2, suffix2, prefix3,
                    suffix3
                );
                Ok(ItemQuality::Rare {
                    first_name,
                    last_name,
                    prefix1,
                    suffix1,
                    prefix2,
                    suffix2,
                    prefix3,
                    suffix3,
                })
            }
            QualityId::Unique => Ok(ItemQuality::Unique { id: reader.read()? }),
            QualityId::Crafted => {
                bitsy_read!(
                    reader, first_name, last_name, prefix1, suffix1, prefix2, suffix2, prefix3,
                    suffix3
                );
                Ok(ItemQuality::Crafted {
                    first_name,
                    last_name,
                    prefix1,
                    suffix1,
                    prefix2,
                    suffix2,
                    prefix3,
                    suffix3,
                })
            }
        }
    }

    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        match self {
            ItemQuality::Low(bits) => writer.write(bits)?,
            ItemQuality::Normal => {}
            ItemQuality::Superior(bits) => writer.write(bits)?,
            ItemQuality::Magic { prefix, suffix } => {
                bitsy_write!(writer, prefix, suffix);
            }
            ItemQuality::Set { id } => writer.write(id)?,
            ItemQuality::Rare {
                first_name,
                last_name,
                prefix1,
                suffix1,
                prefix2,
                suffix2,
                prefix3,
                suffix3,
            } => {
                bitsy_write!(
                    writer, first_name, last_name, prefix1, suffix1, prefix2, suffix2, prefix3,
                    suffix3
                );
            }
            ItemQuality::Unique { id } => writer.write(id)?,
            ItemQuality::Crafted {
                first_name,
                last_name,
                prefix1,
                suffix1,
                prefix2,
                suffix2,
                prefix3,
                suffix3,
            } => {
                bitsy_write!(
                    writer, first_name, last_name, prefix1, suffix1, prefix2, suffix2, prefix3,
                    suffix3
                );
            }
        };
        Ok(())
    }
}
