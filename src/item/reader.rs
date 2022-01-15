use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::bitsy::{BitReader, MyBitVec};
use crate::item::info::ItemDb;
use crate::item::properties::PropertyDb;

pub struct ItemReader {
    reader: BitReader,
    item_db: Rc<dyn ItemDb>,
    property_db: Rc<dyn PropertyDb>,
}

impl ItemReader {
    pub fn new(
        reader: BitReader,
        item_db: Rc<dyn ItemDb>,
        property_db: Rc<dyn PropertyDb>,
    ) -> Self {
        return ItemReader {
            reader,
            item_db,
            property_db,
        };
    }

    pub fn peek_bits(&self, size: usize) -> String {
        return self.reader.peek_bits(size);
    }

    pub fn item_db(&self) -> Rc<dyn ItemDb> {
        Rc::clone(&self.item_db)
    }
    pub fn property_db(&self) -> Rc<dyn PropertyDb> {
        Rc::clone(&self.property_db)
    }
}

impl Deref for ItemReader {
    type Target = BitReader;

    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}

impl DerefMut for ItemReader {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}
