use d2_itemsorter::item::NewItem;
use ratatui::{crossterm::event::Event, layout::Rect, widgets::Paragraph, Frame};

use crate::action::Action;

use super::{
    framed::FramedWindow,
    player::{force_lock, PlayerMutex},
    Window,
};

pub struct ItemsWindow {
    player: PlayerMutex,
    item_indexes: Vec<usize>,
}

impl ItemsWindow {
    pub fn new_framed(player: PlayerMutex) -> FramedWindow<'static> {
        FramedWindow::new(&["Player", "Items"], Box::new(Self::new(player)))
    }

    pub fn new(player: PlayerMutex) -> Self {
        let mut result = Self {
            player,
            item_indexes: Vec::new(),
        };
        result.recalculate_item_indexes();
        result
    }

    fn recalculate_item_indexes(&mut self) {
        let player = force_lock(&self.player);
        let mut items: Vec<(usize, &NewItem)> = player.items.iter().enumerate().collect();
        items.sort_by(|(_, a), (_, b)| {
            a.location
                .value()
                .cmp(&b.location.value())
                .then_with(|| a.y.value().cmp(&b.y.value()))
                .then_with(|| a.x.value().cmp(&b.x.value()))
        });

        self.item_indexes = items.iter().map(|(i, _)| *i).collect();
    }
}

impl Window for ItemsWindow {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        let player = force_lock(&self.player);
        let text = self
            .item_indexes
            .iter()
            .map(|i| {
                let item = &player.items[*i];
                format!(
                    "{}({}) @ ({},{},{})",
                    item.item_info.name,
                    item.item_type.as_string(),
                    item.location.value(),
                    item.y.value(),
                    item.x.value()
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        frame.render_widget(Paragraph::new(text), area);
    }

    fn handle_event(&mut self, event: Event) -> Option<Event> {
        Some(event)
    }

    fn handle_action(&mut self, action: Action) -> Option<Action> {
        Some(action)
    }
}
