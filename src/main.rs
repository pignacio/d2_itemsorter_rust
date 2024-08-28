use std::cmp::min;
use std::rc::Rc;

use bitsy::BitReader;
use clap::Parser;
use item::Item;
use stash::Stash;

use crate::bitsy::BitVecReader;
use crate::item::info::ItemDb;
use crate::item::properties::MapPropertyDb;
use crate::item::reader::ItemReader;

mod bitsy;
mod constants;
mod item;
mod page;
mod player;
mod quality;
mod stash;

#[derive(clap::Parser, Debug)]
struct Args {
    #[arg(long)]
    stash: Option<String>,
    #[arg(long)]
    player: Option<String>,
}

fn main() {
    let args = Args::parse();
    println!("Hello, world! {:?}", args);

    let item_db: Rc<dyn ItemDb> = Rc::new(item::info::MapItemDb::from_data_dir("data/items"));

    println!("{:?}", item_db.get_info("brs "));

    args.stash.map(|path| {
        println!("Reading stash from file '{path}'");
        let bytes = std::fs::read(&path).unwrap();
        let itemreader = ItemReader::new(
            BitVecReader::new(bytes.to_vec()),
            Rc::clone(&item_db),
            Rc::new(MapPropertyDb::new()),
        );
        let stash = Stash::parse(itemreader);

        for page in stash.pages.iter() {
            for item in page.items.iter() {
                if item.simple {
                    println!("{}", item);
                }
            }
        }

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
    });

    args.player.map(|path| {
        println!("Reading player from file '{path}'");
        let bytes = std::fs::read(path).unwrap();
        let mut itemreader = ItemReader::new(
            BitVecReader::new(bytes.to_vec()),
            Rc::clone(&item_db),
            Rc::new(MapPropertyDb::new()),
        );

        println!("Skipped byte: {}", itemreader.read_bits(8));
        println!(
            "Read until item header: {}",
            itemreader.read_until(&constants::ITEM_HEADER)
        );
        //println!("Skipped byte: {}", itemreader.read_bits(8));
        //println!(
        //    "Read until item header: {}",
        //    itemreader.read_until(&constants::ITEM_HEADER)
        //);
        let mut item = Item::parse(&mut itemreader, false);
        println!("Item 1!: {item}");
        item = Item::parse(&mut itemreader, false);
        println!("Item 2!: {item}");

        //let stash = Stash::parse(itemreader);
        //let new_bytes = stash.to_bytes();
        //
        //if bytes.len() != new_bytes.len() {
        //    println!(
        //        "Different byte size: {} vs {}",
        //        bytes.len(),
        //        new_bytes.len()
        //    );
        //}
        //for index in 0..min(bytes.len(), new_bytes.len()) {
        //    let original_byte = bytes[index];
        //    let new_byte = new_bytes[index];
        //    if original_byte != new_byte {
        //        println!(
        //            "Difference at byte #{}: {} vs {}",
        //            index, original_byte, new_byte
        //        );
        //        break;
        //    }
        //}
    });
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
