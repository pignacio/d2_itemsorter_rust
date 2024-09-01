use crate::{
    bitsy::{
        context,
        error::{BitsyError, BitsyErrorExt, BitsyErrorKind},
        impls::{BitsyBytes, BitsyChars, BitsyInt},
        macros::{bitsy_read, bitsy_write},
        result::BitsyResult,
        BitReader, BitSized, BitWriter, Bitsy, MyBitVec,
    },
    item::ItemList,
};

const ATTRIBUTES_HEADER: [u8; 2] = [0x67, 0x66];
const ATTRIBUTE_ID_SIZE: usize = 9;
type AttributeId = BitsyInt<u16, ATTRIBUTE_ID_SIZE>;
const TERMINATOR: u16 = 0b111111111;
const ATTRIBUTE_SIZES: [usize; 16] = [10, 10, 10, 10, 10, 8, 21, 21, 21, 21, 21, 21, 7, 32, 25, 25];
const ATTRIBUTE_NAMES: [&str; 16] = [
    "Strength",
    "Energy",
    "Dexterity",
    "Vitality",
    "Unused stats",
    "Unused skills",
    "Current HP",
    "Max HP",
    "Current MP",
    "Max MP",
    "Current Stamina",
    "Max Stamina",
    "Level",
    "Experience",
    "Gold",
    "Stashed Gold",
];

//#[derive(Debug)]
pub struct Attributes {
    values: Vec<(AttributeId, u32)>,
}

impl std::fmt::Debug for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Attributes");
        for (attribute_id, value) in &self.values {
            debug.field(ATTRIBUTE_NAMES[attribute_id.value() as usize], value);
        }
        debug.finish()
    }
}

impl Bitsy for Attributes {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
        let mut values = Vec::new();
        let header: [u8; 2] = reader.read()?;
        if header != ATTRIBUTES_HEADER {
            return Err(BitsyError::new(
                BitsyErrorKind::InvalidData(format!(
                    "Invalid attributes header {:?} (expected {:?})",
                    header, ATTRIBUTES_HEADER
                )),
                reader.index() - header.bit_size(),
            ));
        }
        loop {
            let attribute_id: AttributeId = reader.read()?;
            if attribute_id.value() == TERMINATOR {
                break;
            } else if attribute_id.value() >= ATTRIBUTE_SIZES.len() as u16 {
                return Err(BitsyError::new(
                    BitsyErrorKind::InvalidData(format!(
                        "Invalid attribute id {}",
                        attribute_id.value()
                    )),
                    reader.index() - attribute_id.bit_size(),
                )
                .prepend_index(values.len()));
            }
            let attribute_size = ATTRIBUTE_SIZES[attribute_id.value() as usize];
            let value = reader.read_int::<u32>(attribute_size)?;
            values.push((attribute_id, value));
        }

        reader.read_padding()?;
        Ok(Attributes { values })
    }

    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        writer.write(&ATTRIBUTES_HEADER)?;
        for (index, (attribute_id, value)) in self.values.iter().enumerate() {
            if attribute_id.value() >= ATTRIBUTE_SIZES.len() as u16 {
                return Err(BitsyError::new(
                    BitsyErrorKind::InvalidData(format!(
                        "Invalid attribute id '{}'",
                        attribute_id.value()
                    )),
                    0,
                )
                .prepend_index(index));
            }
            let attribute_size = ATTRIBUTE_SIZES[attribute_id.value() as usize];
            writer.write(attribute_id)?;
            writer.write_int::<u32>(*value, attribute_size)?;
        }
        writer.write_int(TERMINATOR, ATTRIBUTE_ID_SIZE)?;
        writer.write_padding()?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct Player {
    header: BitsyBytes<4>,
    version: u32,
    file_size: u32,
    checksum: u32,
    active_weapon: u32,
    old_name: BitsyChars<16>,
    status: u8,
    progression: u8,
    unknown1: BitsyBytes<2>,
    class: u8,
    unknown2: BitsyBytes<2>,
    level: u8,
    created_at: u32,
    last_played_at: u32,
    unknown3: BitsyBytes<4>,
    skill_stuff: BitsyBytes<{ 64 + 16 }>,
    appearance: BitsyBytes<41>,
    merc_stuff: BitsyBytes<42>,
    menu_appearance: BitsyBytes<48>,
    new_name: BitsyChars<16>,
    unknown4: BitsyBytes<52>,
    quests: BitsyBytes<298>,
    waypoints: BitsyBytes<80>,
    npcs: BitsyBytes<52>,
    attributes: Attributes,
    skills: BitsyBytes<32>,
    items: ItemList,
}

impl Bitsy for Player {
    fn parse<R: BitReader>(reader: &mut R) -> BitsyResult<Self> {
        bitsy_read!(reader, header, version);
        reader.set_context(&context::VERSION, version);

        bitsy_read!(
            reader,
            file_size,
            checksum,
            active_weapon,
            old_name,
            status,
            progression,
            unknown1,
            class,
            unknown2,
            level,
            created_at,
            last_played_at,
            unknown3,
            skill_stuff,
            appearance,
            merc_stuff,
            menu_appearance,
            new_name,
            unknown4,
            quests,
            waypoints,
            npcs,
            attributes,
            skills,
            items,
        );
        Ok(Self {
            header,
            version,
            file_size,
            checksum,
            active_weapon,
            old_name,
            status,
            progression,
            unknown1,
            class,
            unknown2,
            level,
            created_at,
            last_played_at,
            unknown3,
            skill_stuff,
            appearance,
            merc_stuff,
            menu_appearance,
            new_name,
            unknown4,
            quests,
            waypoints,
            npcs,
            attributes,
            skills,
            items,
        })
    }

    fn write_to<W: BitWriter>(&self, writer: &mut W) -> BitsyResult<()> {
        bitsy_write!(
            writer,
            &self.header,
            &self.version,
            &self.file_size,
            &self.checksum,
            &self.active_weapon,
            &self.old_name,
            &self.status,
            &self.progression,
            &self.unknown1,
            &self.class,
            &self.unknown2,
            &self.level,
            &self.created_at,
            &self.last_played_at,
            &self.unknown3,
            &self.skill_stuff,
            &self.appearance,
            &self.merc_stuff,
            &self.menu_appearance,
            &self.new_name,
            &self.unknown4,
            &self.quests,
            &self.waypoints,
            &self.npcs,
            &self.attributes,
            &self.skills,
            &self.items,
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::bitsy::{BitVecReader, BitVecWriter};

    fn compare_bitvecs(expected: &MyBitVec, actual: &MyBitVec) -> Result<(), String> {
        if expected.len() != actual.len() {
            return Err(format!(
                "BitVec sizes differ! Expected {} ({} bytes) bits, got {} ({} bytes)",
                expected.len(),
                expected.len() / 8,
                actual.len(),
                actual.len() / 8
            ));
        }
        for (index, (expected_bit, actual_bit)) in expected.iter().zip(actual.iter()).enumerate() {
            if expected_bit != actual_bit {
                return Err(format!(
                    "Bit at index {index} (byte {}) differs! Expected {expected_bit}, got {actual_bit}",
                    index /8,
                ));
            }
        }
        Ok(())
    }

    use super::*;
    #[test]
    fn it_works() {
        let bytes = std::fs::read("examples/PlasticSurgeon.d2s").unwrap();
        let bits = MyBitVec::from_vec(bytes);

        let mut reader = BitVecReader::new(bits.clone());
        let player = Player::parse(&mut reader).unwrap();
        println!("Parsed player: {:#?}", player);
        reader.report_next_bytes(32);
        let tail = reader.read_tail().unwrap();
        println!("Tail was {} bits long", tail.len());

        let mut writer = BitVecWriter::new(player.version);
        writer.write(&player).unwrap();
        writer.write_bits(&tail).unwrap();

        let new_bits = writer.into_bits();

        compare_bitvecs(&bits, &new_bits).unwrap();
    }
}
