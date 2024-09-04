use std::cmp::min;
use std::rc::Rc;

pub mod bitsy;
pub mod constants;
pub mod item;
pub mod page;
pub mod player;
pub mod quality;
pub mod stash;

use crate::bitsy::OldBitReader;
use crate::item::info::ItemDb;
use crate::item::properties::MapPropertyDb;
use crate::item::reader::ItemReader;
use crate::stash::Stash;

fn main() {
    println!("Hello, world!");

    let bytes = std::fs::read("stash_example.sss").unwrap();

    let item_db: Rc<dyn ItemDb> = Rc::new(item::info::MapItemDb::from_data_dir("data/items"));

    println!("{:?}", item_db.get_info("brs "));

    let itemreader = ItemReader::new(
        OldBitReader::new(bytes.to_vec()),
        Rc::clone(&item_db),
        Rc::new(MapPropertyDb::new()),
    );
    // let stash = Stash::parse(bytes.to_vec(), item_db);
    let stash = Stash::parse(itemreader);

    let new_bytes = stash.to_bytes();

    // show(stash);

    if bytes.len() != new_bytes.len() {
        println!(
            "Different byte size: {} vs {}",
            bytes.len(),
            new_bytes.len()
        );
    }
    for index in 0..min(bytes.len(), new_bytes.len()) {
        let original_byte = bytes[index];
        let new_byte = new_bytes[index];
        if original_byte != new_byte {
            println!(
                "Difference at byte #{}: {} vs {}",
                index, original_byte, new_byte
            );
            break;
        }
    }
}

pub fn show(stash: Stash) {
    println!("Stash: {}", stash);
    for (index, page) in stash.pages.iter().enumerate() {
        println!(" - Page #{}: {}", index, page);
        for (item_index, item) in page.items.iter().enumerate() {
            println!("   * Item #{}: {}", item_index, item);
        }
    }
}
