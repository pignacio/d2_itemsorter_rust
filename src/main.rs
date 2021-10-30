mod bitsy;
mod constants;
mod item;
mod page;
mod quality;
mod stash;

use stash::Stash;

use std::cmp::min;

fn main() {
    println!("Hello, world!");

    let bytes = std::fs::read("stash_example.sss").unwrap();

    let stash = Stash::parse(bytes.to_vec());

    let new_bytes = stash.to_bytes();

    show(stash);

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

fn show(stash: Stash) {
    println!("Stash: {}", stash);
    for (index, page) in stash.pages.iter().enumerate() {
        println!(" - Page #{}: {}", index, page);
        for (item_index, item) in page.items.iter().enumerate() {
            println!("   * Item #{}: {}", item_index, item);
        }
    }
}







fn arr_to_str(arr: &[u8]) -> String {
    let string = arr
        .iter()
        .map(|value| format!("{}, ", value))
        .collect::<String>();

    return format!("[{}]", string);
}
