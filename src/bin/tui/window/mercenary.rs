use ratatui::{crossterm::event::Event, layout::Rect, widgets::Paragraph, Frame};

use crate::action::Action;

use super::{framed::FramedWindow, Window};

pub struct MercenaryWindow {}

impl MercenaryWindow {
    pub fn new_framed() -> impl Window {
        FramedWindow::new(&["Player", "Mercenary"], Box::new(Self {}))
    }
}

impl Window for MercenaryWindow {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Paragraph::new("Mercenary!"), area);
    }

    fn handle_event(&mut self, event: Event) -> Option<Event> {
        Some(event)
    }

    fn handle_action(&mut self, action: Action) -> Option<Action> {
        Some(action)
    }
}
