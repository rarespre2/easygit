use ratatui::widgets::Widget;

use crate::regions::Region;

use super::panel::PanelBlock;

pub type StashesPanel<W = super::panel::Empty> = PanelBlock<W>;

pub fn panel(selected: bool) -> StashesPanel {
    PanelBlock::new(Region::Stashes, selected)
}

pub fn panel_with_child<W: Widget>(selected: bool, child: W) -> StashesPanel<W> {
    PanelBlock::with_child(Region::Stashes, selected, child)
}
