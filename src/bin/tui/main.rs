use std::{
    io::{self, stdout},
    ops::{Deref, DerefMut},
};

use action::Action;
use d2_itemsorter::{
    bitsy::{BitReader, BitVecReader, MyBitVec},
    item::info::MapItemDb,
    player::Player,
};
use ratatui::{
    crossterm::{
        event,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    widgets::{Clear, Paragraph},
};
use style::Styled;
use window::{player::PlayerWindow, Window};

mod action;
mod window;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let bytes = std::fs::read("examples/LaCopeFull.d2s")?;
    let mut reader = BitVecReader::new(
        MyBitVec::from_vec(bytes),
        MapItemDb::from_data_dir("data/items"),
    );
    let player = reader.read()?;
    {
        let mut terminal = ratatui::init();

        let mut state = UiState::new(player);

        while !state.should_quit() {
            terminal.draw(|f| state.ui(f))?;
            state.handle_events()?;
        }
    }
    Ok(())
}

struct UiState {
    window_stack: Vec<Box<dyn Window>>,
}

impl UiState {
    fn should_quit(&self) -> bool {
        self.window_stack.is_empty()
    }
}

fn breadcrumbs(path: Vec<Span<'_>>) -> Line<'_> {
    let mut spans: Vec<Span<'_>> = path
        .into_iter()
        .flat_map(|span| {
            vec![
                " ".into(),
                span,
                " ".into(),
                Span::styled(">", Style::default().fg(Color::Blue)),
            ]
        })
        .collect();
    // Pop the last >
    spans.pop();
    Line::default().spans(spans)
    //path.iter()
    //    .map(|crumb| format!(" {crumb} "))
    //    .collect::<Vec<String>>()
    //    .join(">")
}

impl UiState {
    fn new(player: Player) -> Self {
        Self {
            window_stack: vec![Box::new(PlayerWindow::new_framed(player))],
        }
    }

    fn ui(&mut self, frame: &mut Frame) {
        frame.render_widget(Clear, frame.area());
        //frame.render_widget(
        //    Paragraph::new(format!("Frame: {}", frame.area()))
        //        .style(Style::default().bg(Color::Red)),
        //    Rect::new(0, 0, 50, 5), //.intersection(frame.area()),
        //);
        if let Some(window) = self.window_stack.last_mut() {
            window.render(frame, frame.area())
        }

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
            let window = self.window_stack.last_mut();
            let action = Action::ProcessEvent(event);

            let residual_action = if let Some(window) = window {
                window.handle_action(action)
            } else {
                Some(action)
            };

            if let Some(action) = residual_action {
                match action {
                    Action::ProcessEvent(_) => {}
                    Action::PopWindowStack => {
                        if !self.window_stack.is_empty() {
                            self.window_stack.pop();
                        }
                    }
                    Action::PushWindowStack(w) => self.window_stack.push(w),
                }
            }
        }
        Ok(())
    }
}
