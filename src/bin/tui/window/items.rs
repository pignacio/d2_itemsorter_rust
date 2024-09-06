use ratatui::{crossterm::event::Event, layout::Rect, widgets::Paragraph, Frame};

use crate::action::Action;

use super::{framed::FramedWindow, Window};

pub struct ItemsWindow {}

impl ItemsWindow {
    pub fn new_framed() -> impl Window {
        FramedWindow::new(&["Player", "Items"], Box::new(Self {}))
    }
}

impl Window for ItemsWindow {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(Paragraph::new("Items!"), area);
    }

    fn handle_event(&mut self, event: Event) -> Option<Event> {
        Some(event)
    }

    fn handle_action(&mut self, action: Action) -> Option<Action> {
        Some(action)
    }
}
