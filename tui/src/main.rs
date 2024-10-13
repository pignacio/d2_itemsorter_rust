use cursive::{
    event::{Event, Key},
    logger,
    reexports::log,
    theme::{BorderStyle, Color, Palette, PaletteColor, Theme},
    view::Resizable,
    views::stack_view::NoShadow,
};
use d2_itemsorter::{
    bitsy::{BitReader, BitVecReader, MyBitVec},
    item::info::MapItemDb,
    player::Player,
};

mod views;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn from_catppuccin(color: catppuccin::Color) -> Color {
    Color::Rgb(color.rgb.r, color.rgb.g, color.rgb.b)
}

fn main() -> Result<()> {
    if std::env::var("CURSIVE_LOG").is_err() {
        std::env::set_var("CURSIVE_LOG", "info");
    }
    let bytes = std::fs::read("examples/LaCopeFull.d2s")?;
    let mut reader = BitVecReader::new(
        MyBitVec::from_vec(bytes),
        MapItemDb::from_data_dir("data/items"),
    );

    let player: Player = reader.read()?;

    logger::set_filter_levels_from_env();
    logger::init();
    log::info!("Starting!");

    let mocha_colors = catppuccin::PALETTE.mocha.colors;
    let mut palette = Palette::default();
    palette[PaletteColor::Background] = from_catppuccin(mocha_colors.base);
    palette[PaletteColor::Shadow] = from_catppuccin(mocha_colors.surface1);
    palette[PaletteColor::View] = from_catppuccin(mocha_colors.surface0);
    palette[PaletteColor::Primary] = from_catppuccin(mocha_colors.text);
    palette[PaletteColor::Secondary] = from_catppuccin(mocha_colors.pink);
    palette[PaletteColor::Tertiary] = from_catppuccin(mocha_colors.peach);
    palette[PaletteColor::TitlePrimary] = from_catppuccin(mocha_colors.mauve);
    palette[PaletteColor::TitleSecondary] = from_catppuccin(mocha_colors.blue);
    palette[PaletteColor::Highlight] = from_catppuccin(mocha_colors.blue);
    palette[PaletteColor::HighlightInactive] = from_catppuccin(mocha_colors.overlay0);
    palette[PaletteColor::HighlightText] = from_catppuccin(mocha_colors.crust);

    let mut siv = cursive::default();
    siv.set_theme(Theme {
        shadow: true,
        borders: BorderStyle::Outset,
        palette,
    });
    siv.add_global_callback(Event::Key(Key::Esc), |s| s.quit());
    siv.add_global_callback(Event::Key(Key::F1), |s| s.toggle_debug_console());

    let player_view = views::PlayerView::new(player);
    siv.screen_mut()
        .add_layer(NoShadow(player_view.full_screen()));

    siv.run();
    Ok(())
}
