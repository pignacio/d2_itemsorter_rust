pub mod bitsy;
pub mod constants;
pub mod item;
pub mod page;
pub mod player;
pub mod quality;
pub mod stash;

use crate::stash::Stash;

pub fn show(stash: Stash) {
    println!("Stash: {}", stash);
    for (index, page) in stash.pages.iter().enumerate() {
        println!(" - Page #{}: {}", index, page);
        for (item_index, item) in page.items.iter().enumerate() {
            println!("   * Item #{}: {}", item_index, item);
        }
    }
}
