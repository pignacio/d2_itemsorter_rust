use std::io::{self, stdout};

use d2_itemsorter::{
    bitsy::{BitReader, BitVecReader, MyBitVec},
    player::Player,
};
use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    widgets::{Block, Paragraph},
};
use window::{PlayerWindow, Window};

mod window;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let bytes = std::fs::read("examples/LaCopeFull.d2s")?;
    let mut reader = BitVecReader::new(MyBitVec::from_vec(bytes));
    let player = reader.read()?;

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut state = UiState::new(player);

    while !state.quit {
        terminal.draw(|f| state.ui(f))?;
        state.handle_events()?;
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

struct UiState {
    window: PlayerWindow,
    quit: bool,
}

fn breadcrumbs(path: &[&str]) -> String {
    path.iter()
        .map(|crumb| format!(" {crumb} "))
        .collect::<Vec<String>>()
        .join(">")
}

impl UiState {
    fn new(player: Player) -> Self {
        Self {
            window: PlayerWindow::new(player),
            quit: false,
        }
    }

    fn ui(&mut self, frame: &mut Frame) {
        self.window.render(frame, frame.area());

        //frame.render_widget(
        //    Paragraph::new(self.player.name().to_string())
        //        .block(Block::bordered().title(breadcrumbs(&["Player"]))),
        //    frame.area(),
        //);
        //
        //frame.render_widget(
        //    Paragraph::new("This is a test!\nAnotherLine!")
        //        .block(Block::bordered().title(breadcrumbs(&["Player", "Test"]))),
        //    Rect::new(10, 10, 30, 3),
        //);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(50))? {
            let event = event::read()?;
            if let Some(event) = self.window.handle_event(event) {
                match event {
                    Event::Key(KeyEvent {
                        kind: KeyEventKind::Press,
                        code: KeyCode::Char('q'),
                        ..
                    }) => {
                        self.quit = true;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}
