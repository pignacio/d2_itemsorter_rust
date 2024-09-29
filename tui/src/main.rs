use cursive::{
    direction::Orientation,
    event::{Event, Key},
    logger,
    reexports::log,
    view::Resizable,
    views::{DebugView, EditView, FixedLayout, LinearLayout, PaddedView, TextView},
    Rect, View,
};
use d2_itemsorter::{
    bitsy::{BitReader, BitVecReader, MyBitVec},
    item::info::MapItemDb,
    player::Player,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let bytes = std::fs::read("examples/LaCopeFull.d2s")?;
    let mut reader = BitVecReader::new(
        MyBitVec::from_vec(bytes),
        MapItemDb::from_data_dir("data/items"),
    );

    let player: Player = reader.read()?;

    logger::init();
    log::info!("Testing!");

    let mut siv = cursive::default();
    siv.add_global_callback(Event::Key(Key::Esc), |s| s.quit());
    siv.add_global_callback(Event::Key(Key::F1), |s| s.toggle_debug_console());

    let layer = LinearLayout::new(Orientation::Vertical)
        .child(PaddedView::lrtb(
            0,
            0,
            0,
            1,
            labeled(
                "Name",
                EditView::new()
                    .content(player.name().trim_matches('\0'))
                    .filler(' ')
                    .max_content_width(16)
                    .min_width(16),
            ),
        ))
        .child(edit("Age:", "18"))
        .child(edit("Test:", ""));

    siv.add_layer(layer.full_screen());

    siv.run();
    Ok(())
}

fn edit(text: &str, initial_value: &str) -> impl View {
    LinearLayout::horizontal()
        .child(TextView::new(format!("{text}")))
        .child(EditView::new().content(initial_value).min_width(5))
}

fn labeled(label: &str, view: impl View) -> impl View {
    LinearLayout::horizontal()
        .child(TextView::new(format!("{label}:")))
        .child(view)
}
