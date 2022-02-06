use crate::bitsy::*;
use std::fmt::{Display, Formatter};

#[derive(Clone)]
pub enum EnumQuality {
    Normal {
        id: u8,
    },
    Low {
        low_quality_type: u8,
    },
    High {
        high_quality_type: u8,
    },
    Magic {
        prefix: u16,
        suffix: u16,
    },
    Set {
        set_id: u16,
    },
    RareOrCrafted {
        id: u8,
        first_name: u8,
        last_name: u8,
        prefix1: Option<u16>,
        prefix2: Option<u16>,
        prefix3: Option<u16>,
        suffix1: Option<u16>,
        suffix2: Option<u16>,
        suffix3: Option<u16>,
    },
    Unique {
        unique_id: u16,
    },
}

const LOW_QUALITY_ID: u8 = 1;
const HIGH_QUALITY_ID: u8 = 3;
const MAGIC_QUALITY_ID: u8 = 4;
pub const SET_QUALITY_ID: u8 = 5;
const RARE_QUALITY_ID: u8 = 6;
const UNIQUE_QUALITY_ID: u8 = 7;
const CRAFTED_QUALITY_ID: u8 = 8;

impl EnumQuality {
    pub fn quality_id(&self) -> u8 {
        match self {
            &EnumQuality::Normal { id } => id,
            &EnumQuality::Low { .. } => LOW_QUALITY_ID,
            &EnumQuality::High { .. } => HIGH_QUALITY_ID,
            &EnumQuality::Magic { .. } => MAGIC_QUALITY_ID,
            &EnumQuality::Set { .. } => SET_QUALITY_ID,
            &EnumQuality::RareOrCrafted { id, .. } => id,
            &EnumQuality::Unique { .. } => UNIQUE_QUALITY_ID,
        }
    }

    pub fn write_quality_bytes(&self, bits: &mut MyBitVec) {
        match self {
            &EnumQuality::Normal { .. } => {}
            &EnumQuality::Low { low_quality_type } => bits.append_int(low_quality_type, 3),
            &EnumQuality::High { high_quality_type } => bits.append_int(high_quality_type, 3),
            &EnumQuality::Magic { prefix, suffix } => {
                bits.append_int(prefix, 11);
                bits.append_int(suffix, 11);
            }
            &EnumQuality::Set { set_id } => {
                bits.append_int(set_id, 12);
            }
            &EnumQuality::RareOrCrafted {
                id,
                first_name,
                last_name,
                prefix1,
                suffix1,
                prefix2,
                suffix2,
                prefix3,
                suffix3,
            } => {
                bits.append_int(first_name, 8);
                bits.append_int(last_name, 8);
                bits.append_optional_int(prefix1, 11);
                bits.append_optional_int(suffix1, 11);
                bits.append_optional_int(prefix2, 11);
                bits.append_optional_int(suffix2, 11);
                bits.append_optional_int(prefix3, 11);
                bits.append_optional_int(suffix3, 11);
            }
            &EnumQuality::Unique { unique_id } => {
                bits.append_int(unique_id, 12);
            }
        }
    }

    pub fn read_quality_bytes(id: u8, bitreader: &mut BitReader) -> EnumQuality {
        match id {
            LOW_QUALITY_ID => EnumQuality::Low {
                low_quality_type: bitreader.read_int(3),
            },
            HIGH_QUALITY_ID => EnumQuality::High {
                high_quality_type: bitreader.read_int(3),
            },
            MAGIC_QUALITY_ID => EnumQuality::Magic {
                prefix: bitreader.read_int(11),
                suffix: bitreader.read_int(11),
            },
            SET_QUALITY_ID => EnumQuality::Set {
                set_id: bitreader.read_int(12),
            },
            RARE_QUALITY_ID | CRAFTED_QUALITY_ID => EnumQuality::RareOrCrafted {
                id,
                first_name: bitreader.read_int(8),
                last_name: bitreader.read_int(8),
                prefix1: bitreader.read_optional_int(11),
                suffix1: bitreader.read_optional_int(11),
                prefix2: bitreader.read_optional_int(11),
                suffix2: bitreader.read_optional_int(11),
                prefix3: bitreader.read_optional_int(11),
                suffix3: bitreader.read_optional_int(11),
            },
            UNIQUE_QUALITY_ID => EnumQuality::Unique {
                unique_id: bitreader.read_int(12),
            },
            _ => EnumQuality::Normal { id },
        }
    }
}

impl Display for EnumQuality {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            &EnumQuality::Normal { .. } => write!(f, "normal"),
            &EnumQuality::Low { low_quality_type } => write!(f, "low({})", low_quality_type),
            &EnumQuality::High { high_quality_type } => write!(f, "high({})", high_quality_type),
            &EnumQuality::Magic { prefix, suffix } => {
                write!(f, "magic(pre:{}, suf:{})", prefix, suffix)
            }
            &EnumQuality::Set { set_id } => write!(f, "set({})", set_id),
            &EnumQuality::RareOrCrafted {
                id,
                first_name,
                last_name,
                ..
            } => write!(
                f,
                "{}({} {})",
                if id == RARE_QUALITY_ID {
                    "rare"
                } else {
                    "crafted"
                },
                first_name,
                last_name,
            ),
            &EnumQuality::Unique { unique_id } => {
                write!(f, "unique({})", unique_id,)
            }
        }
    }
}
