use std::sync::MutexGuard;
use std::{rc::Rc, sync::Mutex};

use d2_itemsorter::player::{Player, ATTRIBUTE_NAMES};
use ratatui::prelude::*;
use ratatui::{
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind},
    layout::Constraint,
    style::{Color, Style},
    widgets::{Block, Borders, List, ListState, Paragraph},
};

use crate::action::Action;

use super::items::ItemsWindow;
use super::mercenary::MercenaryWindow;
use super::stats::StatsWindow;
use super::{framed::FramedWindow, Window};

#[derive(Debug, Clone, Copy)]
enum Options {
    Stats,
    Items,
    Mercenary,
}

const OPTIONS: [Options; 3] = [Options::Stats, Options::Items, Options::Mercenary];

pub type PlayerMutex = Rc<Mutex<Player>>;

pub fn force_lock(player: &PlayerMutex) -> MutexGuard<'_, Player> {
    player.lock().unwrap_or_else(|err| err.into_inner())
}

pub struct PlayerWindow {
    player: PlayerMutex,
    option_list_state: ListState,
}

impl PlayerWindow {
    pub fn new_framed(player: Player) -> impl Window {
        FramedWindow::new(
            &["Player"],
            Box::new(Self {
                player: Rc::new(Mutex::new(player)),
                option_list_state: ListState::default().with_selected(Some(0)),
            }),
        )
    }
}

impl Window for PlayerWindow {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let player = force_lock(&self.player);

        let areas = Layout::vertical(vec![
            Constraint::Length(3),
            Constraint::Length(player.attributes.len() as u16 + 1),
            Constraint::Min(3),
        ])
        .spacing(1)
        .split(area);

        let name = format!("{}!", player.name());
        let mut name_area = areas[0];
        name_area.width = name.len() as u16 + 2;
        frame.render_widget(
            Paragraph::new(format!("{}!", player.name()))
                .block(Block::default().borders(Borders::ALL).title("Name")),
            name_area,
        );

        let mut attributes: Vec<String> = player
            .attributes
            .get()
            .iter()
            .map(|(id, value)| format!("  {}: {}", ATTRIBUTE_NAMES[id.value() as usize], value))
            .collect();
        attributes.insert(0, "Attributes:".to_string());
        frame.render_widget(Paragraph::new(attributes.join("\n")), areas[1]);

        let list = List::new(["Stats", "Items", "Mercenary"])
            .highlight_symbol(">> ")
            .highlight_style(Style::default().fg(Color::LightGreen));

        frame.render_stateful_widget(list, areas[2], &mut self.option_list_state);
    }

    fn handle_event(&mut self, event: Event) -> Option<Event> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => {
                    self.option_list_state.select_previous();
                    None
                }
                KeyCode::Down => {
                    self.option_list_state.select_next();
                    None
                }
                _ => Some(event),
            },
            _ => Some(event),
        }
    }

    fn handle_action(&mut self, action: Action) -> Option<Action> {
        match action {
            Action::ProcessEvent(ref event) => {
                if let Event::Key(KeyEvent {
                    kind: KeyEventKind::Press,
                    code,
                    ..
                }) = event
                {
                    match code {
                        KeyCode::Up => {
                            self.option_list_state.select_previous();
                            None
                        }
                        KeyCode::Down => {
                            self.option_list_state.select_next();
                            None
                        }
                        KeyCode::Enter => {
                            let window: Option<Box<dyn Window>> =
                                match self.option_list_state.selected() {
                                    Some(0) => Some(Box::new(StatsWindow::new_framed())),
                                    Some(1) => {
                                        Some(Box::new(ItemsWindow::new_framed(self.player.clone())))
                                    }
                                    Some(2) => Some(Box::new(MercenaryWindow::new_framed())),
                                    _ => None,
                                };
                            window.map(Action::PushWindowStack)
                        }
                        _ => Some(action),
                    }
                } else {
                    Some(action)
                }
            }
            _ => Some(action),
        }
    }
}
