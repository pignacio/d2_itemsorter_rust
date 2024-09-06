use ratatui::crossterm::event::Event;

use crate::window::Window;

pub enum Action {
    ProcessEvent(Event),
    PopWindowStack,
    PushWindowStack(Box<dyn Window>),
}
