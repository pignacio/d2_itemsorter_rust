use std::io;

use action::Action;
use d2_itemsorter::{
    bitsy::{BitReader, BitVecReader, MyBitVec},
    item::info::MapItemDb,
    player::Player,
};
use ratatui::{crossterm::event, prelude::*, widgets::Clear};
use window::{player::PlayerWindow, Window};

mod action;
mod render;
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
}

impl UiState {
    fn new(player: Player) -> Self {
        Self {
            window_stack: vec![Box::new(PlayerWindow::new_framed(player))],
        }
    }

    fn ui(&mut self, frame: &mut Frame) {
        frame.render_widget(Clear, frame.area());

        if let Some(window) = self.window_stack.last_mut() {
            window.render(frame, frame.area())
        }
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
