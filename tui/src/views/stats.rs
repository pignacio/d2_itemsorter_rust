use cursive::{
    view::{Resizable, ViewWrapper},
    views::{EditView, LinearLayout, PaddedView, TextView},
    wrap_impl,
};
use d2_itemsorter::player::ATTRIBUTE_NAMES;

use super::player::PlayerRef;

pub struct StatsView {
    player: PlayerRef,
    view: LinearLayout,
}

impl StatsView {
    pub fn new(player: PlayerRef) -> Self {
        let view = Self::build_layout(player.clone());
        Self { player, view }
    }

    fn build_layout(player: PlayerRef) -> LinearLayout {
        let data = player.force_lock();
        let mut view = LinearLayout::vertical();
        view.add_child(PaddedView::lrtb(
            0,
            0,
            1,
            1,
            TextView::new(format!("Name: {}", data.name())),
        ));
        for (attribute_id, value) in data.attributes.get() {
            view.add_child(
                LinearLayout::horizontal()
                    .child(TextView::new(format!(
                        "{}: ",
                        ATTRIBUTE_NAMES[attribute_id.value() as usize]
                    )))
                    .child(EditView::new().content(value.to_string()).min_width(10)),
            );
        }
        view
    }
}

impl ViewWrapper for StatsView {
    wrap_impl!(self.view: LinearLayout);
}
