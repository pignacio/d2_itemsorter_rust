use cursive::{
    direction::Orientation,
    event::{Event, Key},
    logger,
    reexports::log,
    theme::{BorderStyle, Color, Palette, PaletteColor, PaletteStyle, Theme},
    view::Resizable,
    views::{stack_view::NoShadow, EditView, LinearLayout, PaddedView, TextView},
    View,
};
use d2_itemsorter::{
    bitsy::{BitReader, BitVecReader, MyBitVec},
    item::info::MapItemDb,
    player::Player,
};

mod views;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const ROSEWATER: Color = Color::Rgb(245, 224, 220);
const FLAMINGO: Color = Color::Rgb(242, 205, 205);
const PINK: Color = Color::Rgb(245, 194, 231);
const MAUVE: Color = Color::Rgb(203, 166, 247);
const RED: Color = Color::Rgb(243, 139, 168);
const MAROON: Color = Color::Rgb(235, 160, 172);
const PEACH: Color = Color::Rgb(250, 179, 135);
const YELLOW: Color = Color::Rgb(249, 226, 175);
const GREEN: Color = Color::Rgb(166, 227, 161);
const TEAL: Color = Color::Rgb(148, 226, 213);
const SKY: Color = Color::Rgb(137, 220, 235);
const SAPPHIRE: Color = Color::Rgb(116, 199, 236);
const BLUE: Color = Color::Rgb(137, 180, 250);
const LAVENDER: Color = Color::Rgb(180, 190, 254);
const TEXT: Color = Color::Rgb(205, 214, 244);
const SUBTEXT_1: Color = Color::Rgb(186, 194, 222);
const SUBTEXT_0: Color = Color::Rgb(166, 173, 200);
const OVERLAY_2: Color = Color::Rgb(147, 153, 178);
const OVERLAY_1: Color = Color::Rgb(127, 132, 156);
const OVERLAY_0: Color = Color::Rgb(108, 112, 134);
const SURFACE_2: Color = Color::Rgb(88, 91, 112);
const SURFACE_1: Color = Color::Rgb(69, 71, 90);
const SURFACE_0: Color = Color::Rgb(49, 50, 68);
const BASE: Color = Color::Rgb(30, 30, 46);
const MANTLE: Color = Color::Rgb(24, 24, 37);
const CRUST: Color = Color::Rgb(17, 17, 27);

fn main() -> Result<()> {
    let bytes = std::fs::read("examples/LaCopeFull.d2s")?;
    let mut reader = BitVecReader::new(
        MyBitVec::from_vec(bytes),
        MapItemDb::from_data_dir("data/items"),
    );

    let player: Player = reader.read()?;

    logger::init();
    log::info!("Testing!");

    let mut palette = Palette::default();
    palette[PaletteColor::Background] = BASE;
    palette[PaletteColor::Shadow] = SURFACE_1;
    palette[PaletteColor::View] = SURFACE_0;
    palette[PaletteColor::Primary] = TEXT;
    palette[PaletteColor::Secondary] = PINK;
    palette[PaletteColor::Tertiary] = PEACH;
    palette[PaletteColor::TitlePrimary] = MAUVE;
    //palette[PaletteColor::TitleSecondary] = Color::Rgb(137, 220, 235);
    palette[PaletteColor::TitleSecondary] = BLUE;
    //palette[PaletteColor::Highlight] = Color::Rgb(137, 180, 250);
    //palette[PaletteColor::Highlight] = Color::Rgb(250, 179, 135);
    palette[PaletteColor::Highlight] = BLUE;
    palette[PaletteColor::HighlightInactive] = OVERLAY_0;
    palette[PaletteColor::HighlightText] = CRUST;

    let mut siv = cursive::default();
    siv.set_theme(Theme {
        shadow: true,
        borders: BorderStyle::Outset,
        palette,
    });
    siv.add_global_callback(Event::Key(Key::Esc), |s| s.quit());
    siv.add_global_callback(Event::Key(Key::F1), |s| s.toggle_debug_console());

    let player_view = views::PlayerView::new(player);
    siv.add_layer(player_view.full_screen());

    siv.run();
    Ok(())
}

fn labeled(label: &str, view: impl View) -> impl View {
    LinearLayout::horizontal()
        .child(TextView::new(format!("{label}:")))
        .child(view)
}
