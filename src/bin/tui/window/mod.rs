use d2_itemsorter::player::Player;
use ratatui::{
    crossterm::event::{Event, KeyCode},
    prelude::*,
    widgets::{Block, Borders, List, ListState, Paragraph},
};

pub trait Window {
    fn render(&mut self, frame: &mut Frame, area: Rect);

    fn handle_event(&mut self, event: Event) -> Option<Event>;
}

fn block(selected: bool) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(if selected {
            Style::default().fg(Color::LightGreen)
        } else {
            Style::default()
        })
}

#[derive(Debug, Clone, Copy)]
enum Options {
    Stats,
    Items,
    Mercenary,
}

const OPTIONS: [Options; 3] = [Options::Stats, Options::Items, Options::Mercenary];

pub(super) struct PlayerWindow {
    player: Player,
    selected_option: usize,
}

impl PlayerWindow {
    pub(super) fn new(player: Player) -> Self {
        Self {
            player,
            selected_option: 0,
        }
    }
}

impl Window for PlayerWindow {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        frame.render_widget(block(false).title(" Player > Test ".to_string()), area);

        frame.render_widget(
            Paragraph::new(format!("Player: {}", self.player.name())),
            Rect::new(2, 2, 50, 1),
        );

        let mut list_state = ListState::default();
        list_state.select(Some(self.selected_option));
        let list = List::new(["Stats", "Items", "Mercenary"])
            .highlight_symbol(">> ")
            .highlight_style(Style::default().fg(Color::LightGreen));

        frame.render_stateful_widget(list, Rect::new(2, 3, 50, 10), &mut list_state);
    }

    fn handle_event(&mut self, event: Event) -> Option<Event> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up => {
                    self.selected_option = if self.selected_option == 0 {
                        OPTIONS.len() - 1
                    } else {
                        self.selected_option - 1
                    };
                    None
                }
                KeyCode::Down => {
                    self.selected_option = (self.selected_option + 1) % OPTIONS.len();
                    None
                }
                _ => Some(event),
            },
            _ => Some(event),
        }
    }
}
