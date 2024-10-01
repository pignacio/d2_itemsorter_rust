use cursive::{
    align::HAlign,
    reexports::log,
    view::{Nameable, Resizable, SizeConstraint, ViewWrapper},
    views::{Dialog, LinearLayout, PaddedView, TextView},
    wrap_impl,
};
use cursive_table_view::{TableView, TableViewItem};
use d2_itemsorter::item::NewItem;

use super::player::PlayerRef;

pub struct ItemsView {
    player: PlayerRef,
    view: LinearLayout,
}

const ITEM_TABLE_VIEW_NAME: &str = "item_table";

impl ItemsView {
    pub fn new(player: PlayerRef) -> Self {
        let items = player
            .force_lock()
            .items
            .iter()
            .map(ItemRow::from)
            .collect();
        log::info!("Items: {:?}", items);
        let submit_player = player.clone();
        let table = TableView::<ItemRow, ItemColumns>::new()
            .column(ItemColumns::Location, "Location", |c| c.width(2))
            .column(ItemColumns::Position, "Position", |c| c.width(7))
            .column(ItemColumns::Code, "Code", |c| c.width(4))
            .column(ItemColumns::Name, "Name", |c| c.align(HAlign::Center))
            .column(ItemColumns::Quality, "Quality", |c| c.align(HAlign::Center))
            .items(items)
            .on_submit(move |siv, _, index| {
                let player = submit_player.force_lock();
                let item = player.items.get(index);

                siv.add_layer(
                    Dialog::new()
                        .title("Item selected!")
                        .content(TextView::new(format!("You selected: {:?}", item)))
                        .button("Ok", |s| {
                            s.pop_layer();
                        }),
                );
            })
            .selected_row(0)
            .with_name(ITEM_TABLE_VIEW_NAME);
        let item_count = player.force_lock().items.len();
        let view = LinearLayout::vertical()
            .child(PaddedView::lrtb(
                1,
                1,
                1,
                1,
                TextView::new(format!("The player has {} items", item_count)),
            ))
            .child(table.resized(SizeConstraint::Full, SizeConstraint::Full));
        Self { player, view }
    }
}

impl ViewWrapper for ItemsView {
    wrap_impl!(self.view: LinearLayout);
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum ItemColumns {
    Location,
    Position,
    Code,
    Name,
    Quality,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ItemRow {
    code: String,
    name: String,
    location: u8,
    position: (u8, u8),
    quality: String,
}

impl ItemRow {
    fn from(item: &NewItem) -> Self {
        Self {
            code: item.item_type.as_string(),
            name: item.item_info.name.to_string(),
            location: item.location.value(),
            position: (item.x.value(), item.y.value()),
            quality: item
                .extended_info
                .as_ref()
                .map(|i| format!("{:?}", i.quality.get_quality_id()))
                .unwrap_or_else(|| "Simple".to_string()),
        }
    }
}

impl TableViewItem<ItemColumns> for ItemRow {
    fn to_column(&self, column: ItemColumns) -> String {
        match column {
            ItemColumns::Location => self.location.to_string(),
            ItemColumns::Position => format!("{}, {}", self.position.0, self.position.1),
            ItemColumns::Code => self.code.to_string(),
            ItemColumns::Name => self.name.to_string(),
            ItemColumns::Quality => self.quality.to_string(),
        }
    }

    fn cmp(&self, other: &Self, column: ItemColumns) -> std::cmp::Ordering
    where
        Self: Sized,
    {
        match column {
            ItemColumns::Location => self.location.cmp(&other.location),
            ItemColumns::Position => self.position.cmp(&other.position),
            ItemColumns::Code => self.code.cmp(&other.code),
            ItemColumns::Name => self.name.cmp(&other.name),
            ItemColumns::Quality => self.quality.cmp(&other.quality),
        }
    }
}
