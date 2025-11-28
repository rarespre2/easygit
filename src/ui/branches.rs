use ratatui::widgets::Widget;

use crate::regions::Region;

use super::panel::PanelBlock;

pub type BranchesPanel<W = super::panel::Empty> = PanelBlock<W>;

pub fn panel(selected: bool) -> BranchesPanel {
    PanelBlock::new(Region::Branches, selected)
}

pub fn panel_with_child<W: Widget>(selected: bool, child: W) -> BranchesPanel<W> {
    PanelBlock::with_child(Region::Branches, selected, child)
}
