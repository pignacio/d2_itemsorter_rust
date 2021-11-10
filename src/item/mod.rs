use crate::bitsy;
use crate::constants;
use std::fmt::{Debug, Display, Formatter};
use std::fs::FileType;

use crate::quality::*;
use bitsy::*;
use bitvec::prelude::*;

pub struct Item {
    header: [u8; 2],
    _unk1: MyBitVec,
    identified: bool,
    _unk2: MyBitVec,
    num_sockets: Option<u8>,
    _unk3: MyBitVec,
    simple: bool,
    ethereal: bool,
    _unk4: MyBitVec,
    inscribed: Option<Vec<u8>>,
    _unk5: MyBitVec,
    runeword: Option<u16>,
    _unk6: MyBitVec,
    x: u8,
    y: u8,
    _unk7: MyBitVec,
    item_type: [u8; 4],
    extended_info: Option<ExtendedInfo>,
    random_pad: Option<[u8; 12]>,
    specific_info: Option<SpecificInfo>,
    item_properties: Option<ItemProperties>,
    // TODO: replace with item
    gems: Vec<u8>,
    tail: MyBitVec,
}

impl Item {
    pub fn parse(bits: &mut BitReader, is_last: bool) -> Item {
        let start = bits.index();
        println!(
            "   * Parsing item. Index:{} Byte:{}",
            start,
            start / 8
        );

        let mut item = Item {
            header: [0; 2],
            _unk1: BitVec::repeat(false, 4),
            identified: false,
            _unk2: BitVec::repeat(false, 6),
            num_sockets: None,
            _unk3: BitVec::repeat(false, 9),
            simple: false,
            ethereal: false,
            _unk4: BitVec::repeat(false, 1),
            inscribed: None,
            _unk5: BitVec::repeat(false, 1),
            runeword: None,
            _unk6: BitVec::repeat(false, 22),
            x: 0,
            y: 0,
            _unk7: BitVec::repeat(false, 3),
            item_type: [0; 4],
            extended_info: None,
            random_pad: None,
            specific_info: None,
            item_properties: None,
            gems: Vec::new(),
            tail: BitVec::new(),
        };

        item.header = bits.read_byte_arr(); // 16
        item._unk1 = bits.read_bits(4); // 20
        item.identified = bits.read_bool(); // 21
        item._unk2 = bits.read_bits(6); // 27
        let socketed = bits.read_bool(); // 28
        if socketed {
            item.num_sockets = Some(0);
        }
        item._unk3 = bits.read_bits(9); // 37
        item.simple = bits.read_bool(); // 38
        item.ethereal = bits.read_bool(); // 39
        item._unk4 = bits.read_bits(1); // 40
        let inscribed = bits.read_bool(); // 41
        item._unk5 = bits.read_bits(1); // 42
        let has_runeword = bits.read_bool(); // 43
        item._unk6 = bits.read_bits(22); // 65
        item.x = bits.read_int(4); // 69
        item.y = bits.read_int(4); // 73
        item._unk7 = bits.read_bits(3); // 76
        item.item_type = bits.read_byte_arr(); // 108
        if !item.simple {
            let (extended_info, gem_count) =
                ExtendedInfo::parse(bits, &mut item, inscribed, has_runeword);
            item.extended_info = Some(extended_info);
            for _ in 0..gem_count {
                item.gems.push(0u8);
            }
        }
        item.random_pad = bits.read_optional_byte_arr();
        if !item.simple {}
        item.tail = bits.read_until(if is_last {
            &constants::PAGE_HEADER
        } else {
            &constants::ITEM_HEADER
        });

        let end = bits.index();
        println!("     Parsed! {} bits:({}:{})", item, start, end);
        let mut written_bits = MyBitVec::new();
        item.append_to(&mut written_bits);

        if written_bits.len() != end - start {
            println!(
                "Different bit count: {} vs {}",
                written_bits.len(),
                end - start
            );
        }
        for index in 0..written_bits.len() {
            let original_bit = bits.get(start + index);
            let new_bit = written_bits[index];
            if original_bit != new_bit {
                println!(
                    "Difference at bit #{}: {} vs {}",
                    index, original_bit, new_bit
                );
                panic!();
            }
        }


        return item;
    }

    pub fn append_to(&self, bitvec: &mut MyBitVec) {
        bitvec.extend_from_raw_slice(&self.header);
        bitvec.append_bits(&self._unk1);
        bitvec.append_bool(self.identified);
        bitvec.append_bits(&self._unk2);
        bitvec.append_bool(self.num_sockets.is_some());
        bitvec.append_bits(&self._unk3);
        bitvec.append_bool(self.simple);
        bitvec.append_bool(self.ethereal);
        bitvec.append_bits(&self._unk4);
        bitvec.append_bool(self.inscribed.is_some());
        bitvec.append_bits(&self._unk5);
        bitvec.append_bool(self.runeword.is_some());
        bitvec.append_bits(&self._unk6);
        bitvec.append_int(self.x as u32, 4);
        bitvec.append_int(self.y as u32, 4);
        bitvec.append_bits(&self._unk7);
        bitvec.extend_from_raw_slice(&self.item_type);
        if let Some(info) = &self.extended_info {
            info.append_to(bitvec, &self);
        }
        bitvec.append_optional_byte_arr(&self.random_pad);

        bitvec.append_bits(&self.tail);
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Item: type({}), {}{}{}{}{}, runeword:{:?}, position:({},{}) extended:{:?} specific:{:?} gems:{} tail is {} bits",
            arr_to_chr(&self.item_type),
            conditional_display(self.identified, "(id)"),
            self.num_sockets.map(|ns| format!("({}os)", ns)).unwrap_or("".to_string()),
            conditional_display(self.simple, "(s)"),
            conditional_display(self.ethereal, "(eth)"),
            conditional_display(self.inscribed.is_some(), "(ins)"),
            self.runeword,
            self.x,
            self.y,
            self.extended_info,
            self.specific_info,
            self.gems.len(),
            self.tail.len()
        )
    }
}

fn conditional_display(condition: bool, display: &str) -> &str {
    return if condition { display } else { "" };
}

fn arr_to_chr(arr: &[u8]) -> String {
    let string = arr.iter().map(|x| *x as char).collect::<String>();

    return format!("[{}]", string);
}

struct ExtendedInfo {
    guid: [u8; 4],
    drop_level: u8,
    quality: Box<dyn Quality>,
    gfx: Option<u8>,
    class_info: Option<MyBitVec>,
}

impl ExtendedInfo {
    pub fn parse(
        bits: &mut BitReader,
        item: &mut Item,
        inscribed: bool,
        has_runeword: bool,
    ) -> (ExtendedInfo, u8) {
        let mut info = ExtendedInfo {
            guid: [0u8; 4],
            drop_level: 0,
            quality: Box::new(NormalQuality::default()),
            gfx: None,
            class_info: None,
        };

        let gem_count: u8 = bits.read_int(3); // 3 (111)
        info.guid = bits.read_byte_arr(); // 35 (143)
        info.drop_level = bits.read_int(7); // 42 (150)
        let quality_id: u8 = bits.read_int(4); // 46 (154)
        info.gfx = bits.read_optional_int(3); // 49 (157)
        info.class_info = bits.read_optional_bits(11); // 60 (168)
        info.quality = ExtendedInfo::parse_quality(quality_id, bits); // ?? (???)
        if has_runeword {
            item.runeword = Some(bits.read_int(16));
        }
        if inscribed {
            panic!("We do not support inscribed items yet");
        }

        return (info, gem_count);
    }

    pub fn append_to(&self, bitvec: &mut MyBitVec, item: &Item) {
        bitvec.append_int(item.gems.len() as u8, 3);
        bitvec.extend_from_raw_slice(&self.guid);
        bitvec.append_int(self.drop_level, 7);
        bitvec.append_int(self.quality.quality_id(), 4);
        bitvec.append_optional_int(self.gfx, 3);
        bitvec.append_optional_bits(&self.class_info);
        self.quality.write_quality_bytes(bitvec);
        if let Some(runeword) = item.runeword {
            bitvec.append_int(runeword, 16);
        }
    }

    fn parse_quality(quality_id: u8, bits: &mut BitReader) -> Box<dyn Quality> {
        match quality_id {
            1 => LowQuality::read_quality_bytes(quality_id, bits),
            3 => HighQuality::read_quality_bytes(quality_id, bits),
            4 => MagicQuality::read_quality_bytes(quality_id, bits),
            5 => SetQuality::read_quality_bytes(quality_id, bits),
            6 => RareQuality::read_quality_bytes(quality_id, bits),
            7 => UniqueQuality::read_quality_bytes(quality_id, bits),
            8 => RareQuality::read_quality_bytes(quality_id, bits),
            _ => NormalQuality::read_quality_bytes(quality_id, bits),
        }
    }
}

impl Display for ExtendedInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "guid:{}, iLvl:{}, q:{}, gfx:{:?} class_info:{:?}",
            arr_to_str(&self.guid),
            self.drop_level,
            self.quality,
            self.gfx,
            self.class_info,
        );
    }
}

impl Debug for ExtendedInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

fn arr_to_str(arr: &[u8]) -> String {
    let string = arr
        .iter()
        .map(|value| format!("{}, ", value))
        .collect::<String>();

    return format!("[{}]", string);
}

#[derive(Debug)]
struct SpecificInfo {
    defense: u16,
    max_durability: u16,
    current_durability: u16,
    quantity: u16,
}

impl SpecificInfo {
    fn parse(bits: &mut BitReader, item: &mut Item, socketed: bool) {
        let mut info = SpecificInfo {
            defense: 0,
            max_durability: 0,
            current_durability: 0,
            quantity: 0,
        };

        info.defense = bits.read_int(11);
        info.max_durability = bits.read_int(9);
        info.current_durability = bits.read_int(9);
        if socketed {
            item.num_sockets = Some(bits.read_int(4));
        }
        info.quantity = bits.read_int(9);
    }

    fn append_to(&self, _bitvec: &mut MyBitVec, _item: &Item) {}
}

impl Display for SpecificInfo {
    fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

struct ItemProperties {
    set_properties: SetProperties,
    properties: Vec<Box<dyn Property>>,
}

struct SetProperties {}

trait Property {}
