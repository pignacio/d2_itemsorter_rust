use std::fmt::{Display, Formatter};
use crate::bitsy::*;
use crate::item::Item;
use crate::constants;
use bitvec::prelude::BitVec;

pub struct Page {
    header: [u8; 5],
    item_count: u16,
    pub items: Vec<Item>,
    tail: MyBitVec,
}

impl Page {
    pub fn parse(bits: &mut BitReader) -> Page {
        let mut page = Page {
            header: [0; 5],
            item_count: 0,
            items: Vec::new(),
            tail: BitVec::new(),
        };

        page.header = bits.read_byte_arr();
        page.item_count = bits.read_u16();
        println!(
            "Parsing page with {} items. Index:{} Byte:{}",
            page.item_count,
            bits.index(),
            bits.index() / 8
        );
        for index in 0..page.item_count {
            let last: bool = index == page.item_count - 1;
            page.items.push(Item::parse(bits, last));
        }
        page.tail = bits.read_until(&constants::PAGE_HEADER);

        return page;
    }

    pub fn append_to(&self, bitvec: &mut MyBitVec) {
        bitvec.extend_from_raw_slice(&self.header);
        bitvec.append_u16(self.item_count);
        for item in &self.items {
            item.append_to(bitvec);
        }
        bitvec.append_bitvec(&self.tail);
    }
}

impl Display for Page {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "I'm a page with {} items. Tail has {} bits",
            self.item_count,
            self.tail.len()
        );
    }
}
