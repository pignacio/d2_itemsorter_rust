use std::fmt::{Display, Formatter};
use crate::bitsy;
use crate::constants;

use bitsy::*;
use bitvec::prelude::*;

type u4 = u8;


pub struct Item {
    header: [u8; 2],
    _unk1: MyBitVec,
    identified: bool,
    _unk2: MyBitVec,
    socketed: bool,
    _unk3: MyBitVec,
    simple: bool,
    ethereal: bool,
    _unk4: MyBitVec,
    inscribed: bool,
    _unk5: MyBitVec,
    has_runeword: bool,
    _unk6: MyBitVec,
    x: u4,
    y: u4,
    _unk7: MyBitVec,
    item_type: [u8; 4],
    tail: MyBitVec,
}

impl Item {
    pub fn parse(bits: &mut BitReader, is_last: bool) -> Item {
        let mut item = Item {
            header: [0; 2],
            _unk1: BitVec::repeat(false, 4),
            identified: false,
            _unk2: BitVec::repeat(false, 6),
            socketed: false,
            _unk3: BitVec::repeat(false, 9),
            simple: false,
            ethereal: false,
            _unk4: BitVec::repeat(false, 1),
            inscribed: false,
            _unk5: BitVec::repeat(false, 1),
            has_runeword: false,
            _unk6: BitVec::repeat(false, 22),
            x: 0,
            y: 0,
            _unk7: BitVec::repeat(false, 3),
            item_type: [0; 4],
            tail: BitVec::new(),
        };

        item.header = bits.read_byte_arr();
        item._unk1 = bits.read_bitvec(4);
        item.identified = bits.read_bool();
        item._unk2 = bits.read_bitvec(6);
        item.socketed = bits.read_bool();
        item._unk3 = bits.read_bitvec(9);
        item.simple = bits.read_bool();
        item.ethereal = bits.read_bool();
        item._unk4 = bits.read_bitvec(1);
        item.inscribed = bits.read_bool();
        item._unk5 = bits.read_bitvec(1);
        item.has_runeword = bits.read_bool();
        item._unk6 = bits.read_bitvec(22);
        item.x = bits.read_int(4) as u4;
        item.y = bits.read_int(4) as u4;
        item._unk7 = bits.read_bitvec(3);
        item.item_type = bits.read_byte_arr();
        item.tail = bits.read_until(if is_last { &constants::PAGE_HEADER } else { &constants::ITEM_HEADER });

        return item;
    }

    pub fn append_to(&self, bitvec: &mut MyBitVec) {
        bitvec.extend_from_raw_slice(&self.header);
        bitvec.append_bitvec(&self._unk1);
        bitvec.append_bool(self.identified);
        bitvec.append_bitvec(&self._unk2);
        bitvec.append_bool(self.socketed);
        bitvec.append_bitvec(&self._unk3);
        bitvec.append_bool(self.simple);
        bitvec.append_bool(self.ethereal);
        bitvec.append_bitvec(&self._unk4);
        bitvec.append_bool(self.inscribed);
        bitvec.append_bitvec(&self._unk5);
        bitvec.append_bool(self.has_runeword);
        bitvec.append_bitvec(&self._unk6);
        bitvec.append_int(self.x as u32, 4);
        bitvec.append_int(self.y as u32, 4);
        bitvec.append_bitvec(&self._unk7);
        bitvec.extend_from_raw_slice(&self.item_type);
        bitvec.append_bitvec(&self.tail);
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Item: type({}), id:{}, sock:{}, simple:{} eth:{} insc:{}, runeword:{}, position:({},{})",
               arr_to_chr(&self.item_type),
               self.identified,
               self.socketed,
               self.simple,
               self.ethereal,
               self.inscribed,
               self.has_runeword,
               self.x,
               self.y,
        )
    }
}

fn arr_to_chr(arr: &[u8]) -> String {
    let string = arr.iter().map(|x| *x as char).collect::<String>();

    return format!("[{}]", string);
}