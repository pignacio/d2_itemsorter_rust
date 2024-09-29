use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind},
    prelude::*,
    widgets::{Block, Borders},
};

use crate::{action::Action, breadcrumbs};

use super::Window;

pub struct FramedWindow<'a> {
    breadcrumbs: Line<'a>,
    window: Box<dyn Window>,
}

impl<'a> FramedWindow<'a> {
    pub fn new(crumbs: &[&'a str], window: Box<dyn Window>) -> Self {
        let crumbs: Vec<Span<'a>> = crumbs.iter().map(|s| Span::raw(s.to_string())).collect();
        Self {
            breadcrumbs: breadcrumbs(crumbs),
            window,
        }
    }
}

impl<'a> Window for FramedWindow<'a> {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::default()
                .borders(Borders::ALL)
                .title(self.breadcrumbs.clone()),
            area,
        );

        self.window.render(
            frame,
            Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2),
        );
    }

    fn handle_event(&mut self, event: Event) -> Option<Event> {
        self.window.handle_event(event)
    }

    fn handle_action(&mut self, action: Action) -> Option<Action> {
        if let Some(residual) = self.window.handle_action(action) {
            match residual {
                Action::ProcessEvent(ref event) => match event {
                    Event::Key(KeyEvent {
                        kind: KeyEventKind::Press,
                        code,
                        ..
                    }) => match code {
                        KeyCode::Esc => return Some(Action::PopWindowStack),
                        _ => {}
                    },
                    _ => {}
                },
                _ => {}
            }
            return Some(residual);
        }
        None
    }
}
