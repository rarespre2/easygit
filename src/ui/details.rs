use ratatui::{buffer::Buffer, layout::Rect, text::Line, widgets::Paragraph, widgets::Widget};

use crate::{git::Commit, regions::Region};

use super::panel::PanelBlock;

pub type DetailsPanel<W = super::panel::Empty> = PanelBlock<W>;

pub fn panel(selected: bool) -> DetailsPanel {
    PanelBlock::new(Region::Details, selected)
}

pub fn panel_with_child<W: Widget>(selected: bool, child: W) -> DetailsPanel<W> {
    PanelBlock::with_child(Region::Details, selected, child)
}

pub struct DetailsView<'a> {
    commit: Option<&'a Commit>,
}

impl<'a> DetailsView<'a> {
    pub fn new(commit: Option<&'a Commit>) -> Self {
        Self { commit }
    }
}

impl Widget for DetailsView<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let message = match self.commit {
            Some(commit) => format!("Details for \"{}\" are not implemented yet", commit.summary),
            None => "Select a commit to view details".to_string(),
        };
        Paragraph::new(Line::from(message)).render(area, buf);
    }
}
