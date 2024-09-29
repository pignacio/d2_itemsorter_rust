use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind},
    layout::Rect,
    widgets::Paragraph,
    Frame,
};

use crate::{
    action::Action,
    render::{Block, CompAction, Component, Group, Text},
};

use super::{framed::FramedWindow, Window};

pub struct StatsWindow {
    group: Group,
}

impl StatsWindow {
    pub fn new_framed() -> impl Window {
        let group = Group::new(
            vec![
                Box::new(Text::new("Stats")),
                Box::new(Text::new("Two\nLiner")),
                Box::new(Block::new(30, 3)),
            ],
            2,
        );
        FramedWindow::new(&["Player", "Stats"], Box::new(Self { group }))
    }
}

impl Window for StatsWindow {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        self.group.render(0, frame, area);
    }

    fn handle_event(&mut self, event: Event) -> Option<Event> {
        Some(event)
    }

    fn handle_action(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::ProcessEvent(Event::Key(KeyEvent {
                code,
                kind: KeyEventKind::Press,
                ..
            })) => match code {
                KeyCode::Up => {
                    self.group
                        .handle_action(CompAction::SelectPrevious)
                        // Wraparound
                        .map(|a| self.group.handle_action(a));
                    None
                }
                KeyCode::Down => {
                    self.group
                        .handle_action(CompAction::SelectNext)
                        // Wraparound
                        .map(|a| self.group.handle_action(a));
                    None
                }
                _ => Some(action),
            },
            _ => Some(action),
        }
    }
}
