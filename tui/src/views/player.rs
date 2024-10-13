use std::{
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard},
};

use cursive::{
    reexports::log,
    view::{Nameable, ViewWrapper},
    views::TextView,
    wrap_impl,
};
use cursive_tabs::TabPanel;
use d2_itemsorter::player::Player;

use super::{ItemsView, StatsView};

#[derive(Clone)]
pub struct PlayerRef {
    arc: Arc<Mutex<Player>>,
}

impl PlayerRef {
    pub fn new(player: Player) -> Self {
        Self {
            arc: Arc::new(Mutex::new(player)),
        }
    }

    pub fn force_lock(&self) -> MutexGuard<Player> {
        self.arc.lock().unwrap_or_else(|e| {
            self.arc.clear_poison();
            e.into_inner()
        })
    }
}

impl Deref for PlayerRef {
    type Target = Arc<Mutex<Player>>;
    fn deref(&self) -> &Self::Target {
        &self.arc
    }
}

pub struct PlayerView {
    player: PlayerRef,
    tabs: TabPanel,
}

impl PlayerView {
    pub fn new(player: Player) -> Self {
        let player = PlayerRef::new(player);
        let mut tabs = TabPanel::new()
            .with_tab(ItemsView::new(player.clone()).with_name("Items"))
            .with_tab(TextView::new("This is the sarasa view!").with_name("Sarasa"))
            // We add the first tab last because adding tabs set the cursor
            .with_tab_at(StatsView::new(player.clone()).with_name("Stats"), 0);

        tabs.set_active_tab("Stats").unwrap();

        Self { player, tabs }
    }

    pub fn test(&mut self) {
        log::info!("I have mutable access to the player MWAHAHA");
    }
}

impl ViewWrapper for PlayerView {
    wrap_impl!(self.tabs: TabPanel);
}
