use ratatui::{crossterm::event::Event, prelude::*};

use crate::action::Action;

pub mod framed;
pub mod items;
pub mod mercenary;
pub mod player;
pub mod stats;

pub trait Window {
    fn render(&mut self, frame: &mut Frame, area: Rect);

    fn handle_event(&mut self, event: Event) -> Option<Event>;
    fn handle_action(&mut self, action: Action) -> Option<Action>;
}
