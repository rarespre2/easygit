use ratatui::widgets::Widget;

use crate::regions::Region;

use super::panel::PanelBlock;

pub type DetailsPanel<W = super::panel::Empty> = PanelBlock<W>;

pub fn panel(selected: bool) -> DetailsPanel {
    PanelBlock::new(Region::Details, selected)
}

pub fn panel_with_child<W: Widget>(selected: bool, child: W) -> DetailsPanel<W> {
    PanelBlock::with_child(Region::Details, selected, child)
}
