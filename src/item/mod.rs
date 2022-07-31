use std::fmt::{Debug, Display, Formatter};

use bitvec::prelude::*;

use bitsy::*;

use crate::bitsy;
use crate::constants;
use crate::item::info::ItemInfo;
use crate::item::properties::PropertyList;
use crate::item::reader::ItemReader;
use crate::quality::*;

pub mod info;
pub mod properties;
pub mod reader;

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
    item_info: ItemInfo,
    extended_info: Option<ExtendedInfo>,
    random_pad: Option<[u8; 12]>,
    specific_info: Option<SpecificInfo>,
    item_properties: Option<ItemProperties>,
    // TODO: replace with item
    gems: Vec<u8>,
    tail: MyBitVec,
}

impl Item {
    pub fn parse(bits: &mut ItemReader, is_last: bool) -> Item {
        let start = bits.index();
        // println!("Item initial  bits:{}", bits.peek_bits(512));
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
            item_info: ItemInfo::default(),
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
        item.item_info = bits
            .item_db()
            .get_info(std::str::from_utf8(&item.item_type).unwrap());
        if !item.simple {
            let (extended_info, gem_count) =
                ExtendedInfo::parse(bits, &mut item, inscribed, has_runeword);
            item.extended_info = Some(extended_info);
            for _ in 0..gem_count {
                item.gems.push(0u8);
            }
        }
        item.random_pad = bits.read_optional_byte_arr();
        if !item.simple {
            item.specific_info = Some(SpecificInfo::parse(bits, &mut item, socketed));
            item.item_properties = Some(ItemProperties::parse(bits, &item));
        }
        item.tail = bits.read_until(if is_last {
            &constants::PAGE_HEADER
        } else {
            &constants::ITEM_HEADER
        });

        let end = bits.index();
        // println!(
        //     "     Parsed! {} bits:({}:{}, len:{})",
        //     item,
        //     start,
        //     end,
        //     end - start
        // );
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

        if let Some(ref properties) = item.item_properties {
            if !properties.properties.tail_is_padding() {
                println!("Tail was not padding for {}", item);
                properties.properties.properties.iter().for_each(|prop| {
                    println!("* {}", prop);
                });
                let tail = &properties.properties.tail;
                println!("Tail has size {}: {}", tail.len(), tail);
                println!("First property id: {}", Item::bits_to_int(tail, 0, 9));
                let mut values = [0; 20];
                for i in 0..values.len() {
                    values[i] = Item::bits_to_int(tail, 9, i + 1);
                }
                println!(
                    " * First possible values: {}",
                    values.map(|x| x.to_string()).join(", ")
                );
            } else if properties
                .properties
                .iter()
                .any(|p| p.definition().id() == 11157)
            {
                println!("Debugging item: {}", item);
                properties.properties.properties.iter().for_each(|prop| {
                    println!("* {}", prop);
                });
            }
        }

        return item;
    }

    fn bits_to_int(bit_vec: &MyBitVec, skip: usize, size: usize) -> i32 {
        if bit_vec.len() <= skip {
            return -1;
        }
        let mut reader = BitReader::new(bit_vec.to_bitvec().into_vec());
        if 0 < skip {
            let _ignored: u32 = reader.read_int(skip);
        }
        reader.read_int(std::cmp::min(size, bit_vec.len() - skip))
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
        self.specific_info
            .as_ref()
            .map(|info| info.append_to(bitvec, &self));
        self.item_properties
            .as_ref()
            .map(|props| props.append_to(bitvec, &self));
        bitvec.append_bits(&self.tail);
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Item: {}({}){}{}{}{}{}{}, pos:({},{}) extended:{} specific:{} gems:{} tail is {} bits ({})",
            self.item_info.name,
            arr_to_chr(&self.item_type),
            conditional_display(!self.identified, "u"),
            self.num_sockets.map(|ns| format!("({}os)", ns)).unwrap_or("".to_string()),
            conditional_display(self.simple, "(s)"),
            conditional_display(self.ethereal, "(eth)"),
            conditional_display(self.inscribed.is_some(), "(ins)"),
            self.runeword.map(|rw| format!("(rw:{})", rw)).unwrap_or("".to_string()),
            self.x,
            self.y,
            self.extended_info.as_ref().map(|info| format!("[{}]", info)).unwrap_or("None".to_string()),
            self.specific_info.as_ref().map(|info| format!("[{}]", info)).unwrap_or("None".to_string()),
            self.gems.len(),
            self.tail.len(),
            if is_padding(&self.tail) { "OK".to_string() } else { format!("NOK ({})", &self.tail[..std::cmp::min(32, self.tail.len())]) }
        )
    }
}

fn is_padding(tail: &MyBitVec) -> bool {
    return tail.len() < 8 && tail.not_any();
}

fn conditional_display(condition: bool, display: &str) -> &str {
    return if condition { display } else { "" };
}

fn arr_to_chr(arr: &[u8]) -> String {
    let string = arr.iter().map(|x| *x as char).collect::<String>();

    return format!("{}", string);
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
            "guid:{}, iLvl:{}, q:{}, gfx:{:?} class_info:{}",
            to_hex(&self.guid),
            self.drop_level,
            self.quality,
            self.gfx,
            self.class_info
                .as_ref()
                .map(|x| format!("{}", x))
                .unwrap_or("None".to_string()),
        );
    }
}

impl Debug for ExtendedInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

fn to_hex(arr: &[u8]) -> String {
    return arr
        .iter()
        .map(|value| format!("{:X?}", value))
        .collect::<String>();
}

#[derive(Debug)]
struct SpecificInfo {
    defense: Option<u16>,
    max_durability: Option<u16>,
    current_durability: Option<u16>,
    quantity: Option<u16>,
}

impl SpecificInfo {
    fn parse(bits: &mut BitReader, item: &mut Item, socketed: bool) -> SpecificInfo {
        let mut info = SpecificInfo {
            defense: None,
            max_durability: None,
            current_durability: None,
            quantity: None,
        };

        if item.item_info.has_defense {
            info.defense = Some(bits.read_int(11));
        }
        if item.item_info.has_durability {
            let max_durability = bits.read_int(9);
            info.max_durability = Some(max_durability);
            if max_durability > 0 {
                info.current_durability = Some(bits.read_int(9));
            }
        }
        if socketed {
            item.num_sockets = Some(bits.read_int(4));
        }
        if item.item_info.has_quantity {
            info.quantity = Some(bits.read_int(9));
        }
        return info;
    }

    fn append_to(&self, bitvec: &mut MyBitVec, item: &Item) {
        self.defense.map(|x| bitvec.append_int(x, 11));
        self.max_durability.map(|x| bitvec.append_int(x, 9));
        self.current_durability.map(|x| bitvec.append_int(x, 9));
        item.num_sockets.map(|x| bitvec.append_int(x, 4));
        self.quantity.map(|x| bitvec.append_int(x, 9));
    }
}

impl Display for SpecificInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "{}{}{}",
            self.defense
                .map(|x| { format!("Def:{} ", x) })
                .unwrap_or(String::default()),
            self.max_durability
                .map(|x| { format!("Dur:{}/{} ", self.current_durability.unwrap_or(0), x) })
                .unwrap_or(String::default()),
            self.quantity
                .map(|x| { format!("Qty:{} ", x) })
                .unwrap_or(String::default()),
        );
    }
}

struct ItemProperties {
    properties: PropertyList,
    set_properties: [Option<PropertyList>; 5],
}

impl ItemProperties {
    pub fn parse(reader: &mut ItemReader, item: &Item) -> Self {
        // println!("Property initial  bits: {}", reader.peek_bits(128));
        let mut has_set_props = [false; 5];
        if ItemProperties::is_set_item(item) {
            // println!("Parsing set properties bits @{}", reader.index());
            has_set_props = has_set_props.map(|_| reader.read_bool());
        }
        // println!("Parsing properties bits @{}", reader.index());
        let properties = PropertyList::parse(reader);
        let set_properties = has_set_props.map(|should_parse| {
            if should_parse {
                // println!("Parsing set properties @{}", reader.index());
                Some(PropertyList::parse(reader))
            } else {
                None
            }
        });

        ItemProperties {
            properties,
            set_properties,
        }
    }

    fn is_set_item(item: &Item) -> bool {
        let quality_id = item
            .extended_info
            .as_ref()
            .map(|info| info.quality.quality_id())
            .unwrap();
        return quality_id == SET_QUALITY_ID;
    }

    pub fn append_to(&self, bit_vec: &mut MyBitVec, item: &Item) {
        if ItemProperties::is_set_item(item) {
            for opt in &self.set_properties {
                bit_vec.append_bool(opt.is_some());
            }
        }
        self.properties.append_to(bit_vec);
        for opt in &self.set_properties {
            opt.as_ref().map(|props| props.append_to(bit_vec));
        }
    }
}
