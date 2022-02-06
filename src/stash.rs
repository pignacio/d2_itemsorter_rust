use std::fmt::{Display, Formatter};

use bitvec::prelude::*;

use bitsy::*;

use crate::bitsy;
use crate::item::reader::ItemReader;
use crate::page::Page;

pub struct Stash {
    header: [u8; 6],
    _unk1: BitArr!(for 32, in MyBitOrder, u8),
    pub pages: Vec<Page>,
    tail: MyBitVec,
}

impl Stash {
    fn new() -> Stash {
        let unk1 = bitarr![MyBitOrder, u8; 0; 32];
        return Stash {
            header: [0; 6],
            _unk1: unk1,
            pages: Vec::new(),
            tail: BitVec::new(),
        };
    }

    pub fn parse(mut itemreader: ItemReader) -> Stash {
        let mut stash = Stash::new();

        stash.header = itemreader.read_byte_arr();
        itemreader.read_into_bitarr(32, &mut stash._unk1);
        let page_count = itemreader.read_int(32);
        println!("Page count: {}", page_count);
        for index in 0..page_count {
            stash.pages.push(Page::parse(&mut itemreader));
            println!(
                "Parsed page {}/{}: {}",
                index + 1,
                page_count,
                stash.pages.last().unwrap()
            )
        }
        stash.tail = itemreader.tail();
        return stash;
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bitvec: MyBitVec = BitVec::new();
        bitvec.extend_from_raw_slice(&self.header);
        bitvec.extend_from_bitslice(&self._unk1);
        bitvec.append_u32(self.pages.len() as u32);
        for page in &self.pages {
            page.append_to(&mut bitvec);
        }
        bitvec.append_bits(&self.tail);
        return bitvec.into_vec();
    }
}

impl Display for Stash {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "I'm a stash with {} pages. Tail has {} bits",
            self.pages.len(),
            self.tail.len()
        );
    }
}
