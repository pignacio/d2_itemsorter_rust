use crate::bitsy::*;
use std::fmt::{Display, Formatter};

pub const SET_QUALITY_ID: u8 = 5;

pub trait Quality: Display {
    fn quality_id(&self) -> u8;
    fn write_quality_bytes(&self, bitvec: &mut MyBitVec);
    fn read_quality_bytes(id: u8, bitreader: &mut BitVecReader) -> Box<dyn Quality>
    where
        Self: Sized;
}

pub struct NormalQuality {
    id: u8,
}

impl NormalQuality {
    pub fn default() -> NormalQuality {
        NormalQuality { id: 15 }
    }
}

impl Quality for NormalQuality {
    fn quality_id(&self) -> u8 {
        self.id
    }

    fn write_quality_bytes(&self, _bitvec: &mut MyBitVec) {
        // No extra bits
    }

    fn read_quality_bytes(id: u8, _bitreader: &mut BitVecReader) -> Box<dyn Quality> {
        Box::new(NormalQuality { id })
    }
}

impl Display for NormalQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "normal")
    }
}

pub struct LowQuality {
    low_quality_type: u8,
}

impl Quality for LowQuality {
    fn quality_id(&self) -> u8 {
        1
    }

    fn write_quality_bytes(&self, bitvec: &mut MyBitVec) {
        bitvec.append_int(self.low_quality_type as u32, 3);
    }

    fn read_quality_bytes(id: u8, bitreader: &mut BitVecReader) -> Box<dyn Quality> {
        assert_eq!(id, 1, "Low quality should have id = 1");
        Box::new(LowQuality {
            low_quality_type: bitreader.read_int(3),
        })
    }
}

impl Display for LowQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "low({})", self.low_quality_type)
    }
}

pub struct HighQuality {
    high_quality_type: u8,
}

impl Quality for HighQuality {
    fn quality_id(&self) -> u8 {
        3
    }

    fn write_quality_bytes(&self, bitvec: &mut MyBitVec) {
        bitvec.append_int(self.high_quality_type as u32, 3);
    }

    fn read_quality_bytes(id: u8, bitreader: &mut BitVecReader) -> Box<dyn Quality> {
        assert_eq!(id, 3, "High quality should have id = 3");
        Box::new(HighQuality {
            high_quality_type: bitreader.read_int(3),
        })
    }
}

impl Display for HighQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "low({})", self.high_quality_type)
    }
}

pub struct MagicQuality {
    prefix: u16,
    suffix: u16,
}

impl Quality for MagicQuality {
    fn quality_id(&self) -> u8 {
        4
    }

    fn write_quality_bytes(&self, bitvec: &mut MyBitVec) {
        bitvec.append_int(self.prefix as u32, 11);
        bitvec.append_int(self.suffix as u32, 11);
    }

    fn read_quality_bytes(id: u8, bitreader: &mut BitVecReader) -> Box<dyn Quality> {
        assert_eq!(id, 4, "Magic quality should have id = 4");
        Box::new(MagicQuality {
            prefix: bitreader.read_int(11),
            suffix: bitreader.read_int(11),
        })
    }
}

impl Display for MagicQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "magic(pre:{}, suf:{})", self.prefix, self.suffix)
    }
}

pub struct SetQuality {
    set_id: u16,
}

impl Quality for SetQuality {
    fn quality_id(&self) -> u8 {
        SET_QUALITY_ID
    }

    fn write_quality_bytes(&self, bitvec: &mut MyBitVec) {
        bitvec.append_int(self.set_id as u32, 12);
    }

    fn read_quality_bytes(id: u8, bitreader: &mut BitVecReader) -> Box<dyn Quality> {
        assert_eq!(id, SET_QUALITY_ID, "Set quality should have id = 5");
        Box::new(SetQuality {
            set_id: bitreader.read_int(12),
        })
    }
}

impl Display for SetQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "set({})", self.set_id)
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
        self.id
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

    fn read_quality_bytes(id: u8, bitreader: &mut BitVecReader) -> Box<dyn Quality> {
        assert!(
            id == 6 || id == 8,
            "Rare/Crafted quality should have id in [6, 8]"
        );
        Box::new(RareQuality {
            id,
            first_name: bitreader.read_int(8),
            last_name: bitreader.read_int(8),
            prefix1: bitreader.read_optional_int(11),
            suffix1: bitreader.read_optional_int(11),
            prefix2: bitreader.read_optional_int(11),
            suffix2: bitreader.read_optional_int(11),
            prefix3: bitreader.read_optional_int(11),
            suffix3: bitreader.read_optional_int(11),
        })
    }
}

impl Display for RareQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({} {})",
            if self.id == 6 { "rare" } else { "crafted" },
            self.first_name,
            self.last_name,
        )
    }
}

pub struct UniqueQuality {
    unique_id: u16,
}

impl Quality for UniqueQuality {
    fn quality_id(&self) -> u8 {
        7
    }

    fn write_quality_bytes(&self, bitvec: &mut MyBitVec) {
        bitvec.append_int(self.unique_id, 12);
    }

    fn read_quality_bytes(id: u8, bitreader: &mut BitVecReader) -> Box<dyn Quality> {
        assert_eq!(id, 7, "Unique quality should have id = 7");
        Box::new(UniqueQuality {
            unique_id: bitreader.read_int(12),
        })
    }
}

impl Display for UniqueQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "unique({})", self.unique_id,)
    }
}
