use std::ops::{Deref, DerefMut};
use std::rc::Rc;

use crate::bitsy::BitReader;
use crate::item::info::ItemDb;

pub struct ItemReader {
    reader: BitReader,
    item_db: Rc<dyn ItemDb>,
}

impl ItemReader {
    pub fn new(reader: BitReader, item_db: Rc<dyn ItemDb>) -> Self {
        return ItemReader { reader, item_db };
    }

    pub fn item_db(&self) -> Rc<dyn ItemDb> {
        Rc::clone(&self.item_db)
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
