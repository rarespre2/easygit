use crate::regions::Region;

use super::panel::PanelBlock;

pub type StashesPanel<W = super::panel::Empty> = PanelBlock<W>;

pub fn panel(selected: bool) -> StashesPanel {
    PanelBlock::new(Region::Stashes, selected)
}
