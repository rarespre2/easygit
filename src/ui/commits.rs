use ratatui::widgets::Widget;

use crate::regions::Region;

use super::panel::PanelBlock;

pub type CommitsPanel<W = super::panel::Empty> = PanelBlock<W>;

pub fn panel(selected: bool) -> CommitsPanel {
    PanelBlock::new(Region::Commits, selected)
}

pub fn panel_with_child<W: Widget>(selected: bool, child: W) -> CommitsPanel<W> {
    PanelBlock::with_child(Region::Commits, selected, child)
}
