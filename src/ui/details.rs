use crate::regions::Region;

use super::panel::PanelBlock;

pub type DetailsPanel<W = super::panel::Empty> = PanelBlock<W>;

pub fn panel(selected: bool) -> DetailsPanel {
    PanelBlock::new(Region::Details, selected)
}
