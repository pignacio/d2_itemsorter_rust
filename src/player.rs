use crate::bitsy::{BitReader, BitVecReader, BitWritable, Bits, Bitsy, MyBitVec};

#[derive(Default)]
struct Player {
    header: [u8; 4],
    version: u32,
    file_size: u32,
    checksum: u32,
    active_weapon: u32,
    old_name: [u8; 16],
    status: u8,
    progression: u8,
    __unk1: Bits<16>,
    class: u8,
    __unk2: Bits<16>,
    level: u8,
    __unk3: Bits<32>,
    last_played: u32,
    __unk4: Bits<32>,
    __skill_stuff: Bits<{ 80 * 8 }>,
    __appearance_stuff: Bits<{ 41 * 8 }>,
    __merc_stuff: Bits<{ 14 * 8 }>,
    __unk5: Bits<{ 76 * 8 }>,
    new_name: [u8; 16],
    __unk6: Bits<{ 52 * 8 }>,
    quest_stuff: Bits<{ 298 * 8 }>,
    waypoints: Bits<{ 80 * 8 }>,
    npc_introductions: Bits<{ 52 * 8 }>,
    attributes: Attributes,
    skills: [u8; 32],
    tail: MyBitVec,
}

impl Player {
    pub fn name(&self) -> String {
        if self.version > 97 {
            String::from_utf8(self.new_name.to_vec()).unwrap()
        } else {
            String::from_utf8(self.old_name.to_vec()).unwrap()
        }
    }
}

impl Bitsy for Player {
    #[allow(clippy::field_reassign_with_default)]
    fn parse<R: BitReader>(reader: &mut R) -> Self {
        let mut player = Player::default();
        player.header = reader.read();
        player.version = reader.read();
        player.file_size = reader.read();
        player.checksum = reader.read();
        player.active_weapon = reader.read();
        player.old_name = reader.read();
        player.status = reader.read();
        player.progression = reader.read();
        player.__unk1 = reader.read();
        player.class = reader.read();
        player.__unk2 = reader.read();
        player.level = reader.read();
        player.__unk3 = reader.read();
        player.last_played = reader.read();
        player.__unk4 = reader.read();
        player.__skill_stuff = reader.read();
        player.__appearance_stuff = reader.read();
        player.__merc_stuff = reader.read();
        player.__unk5 = reader.read();
        player.new_name = reader.read();
        player.__unk6 = reader.read();
        player.quest_stuff = reader.read();
        player.waypoints = reader.read();
        player.npc_introductions = reader.read();
        player.attributes = reader.read();
        player.skills = reader.read();

        reader.report_next_bytes(100);

        player.tail = reader.tail();
        player
    }
}

impl BitWritable for Player {
    fn append_to<W: crate::bitsy::BitWriter>(&self, writer: &mut W) {
        writer.append(&self.header);
        writer.append(&self.version);
        writer.append(&self.file_size);
        writer.append(&self.checksum);
        writer.append(&self.active_weapon);
        writer.append(&self.old_name);
        writer.append(&self.status);
        writer.append(&self.progression);
        writer.append(&self.__unk1);
        writer.append(&self.class);
        writer.append(&self.__unk2);
        writer.append(&self.level);
        writer.append(&self.__unk3);
        writer.append(&self.last_played);
        writer.append(&self.__unk4);
        writer.append(&self.__skill_stuff);
        writer.append(&self.__appearance_stuff);
        writer.append(&self.__merc_stuff);
        writer.append(&self.__unk5);
        writer.append(&self.new_name);
        writer.append(&self.__unk6);
        writer.append(&self.quest_stuff);
        writer.append(&self.waypoints);
        writer.append(&self.npc_introductions);
        writer.append(&self.attributes);
        writer.append(&self.skills);

        writer.append_bits(&self.tail);
    }
}

const ATTRIBUTE_BITS: [u8; 16] = [10, 10, 10, 10, 10, 8, 21, 21, 21, 21, 21, 21, 7, 32, 25, 25];
const ATTRIBUTE_TERMINATOR: u16 = 0b111111111;
const ATTRIBUTES_HEADER: [u8; 2] = [0x67, 0x66];

#[derive(Default)]
struct Attributes {
    attributes: Vec<(u16, u32)>,
}

impl Bitsy for Attributes {
    fn parse<R: BitReader>(reader: &mut R) -> Self {
        let header: [u8; 2] = reader.read();
        if header != ATTRIBUTES_HEADER {
            panic!("Invalid attributes header");
        }
        let mut attributes: Vec<(u16, u32)> = Vec::new();
        loop {
            let attribute_id: u16 = reader.read_int(9);
            if attribute_id == ATTRIBUTE_TERMINATOR {
                break;
            }
            let attribute_value = reader.read_int(ATTRIBUTE_BITS[attribute_id as usize].into());
            attributes.push((attribute_id, attribute_value));
        }
        reader.read_padding();

        Attributes { attributes }
    }
}

impl BitWritable for Attributes {
    fn append_to<W: crate::bitsy::BitWriter>(&self, writer: &mut W) {
        writer.append(&ATTRIBUTES_HEADER);
        for (attribute_id, attribute_value) in &self.attributes {
            writer.append_int(*attribute_id, 9);
            writer.append_int(
                *attribute_value,
                ATTRIBUTE_BITS[*attribute_id as usize].into(),
            );
        }
        writer.append_int(ATTRIBUTE_TERMINATOR, 9);
        writer.append_padding();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        let mut reader = BitVecReader::new(std::fs::read("LaCope.d2s").unwrap());
        let player = Player::parse(&mut reader);
        println!("Version: {:?}", player.version);
        println!("File size: {:?}", player.file_size);
        println!("Checksum: {:?}", player.checksum);
        println!("Name: {:?}", player.name());
    }
}
