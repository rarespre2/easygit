use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};

use crate::regions::Region;

pub struct Empty;

impl Widget for Empty {
    fn render(self, _area: Rect, _buf: &mut Buffer) {}
}

pub struct PanelBlock<W = Empty> {
    region: Region,
    selected: bool,
    child: W,
    footer: Option<Line<'static>>,
}

impl PanelBlock<Empty> {
    pub fn new(region: Region, selected: bool) -> Self {
        Self {
            region,
            selected,
            child: Empty,
            footer: None,
        }
    }
}

impl<W: Widget> PanelBlock<W> {
    pub fn with_child(region: Region, selected: bool, child: W) -> Self {
        Self {
            region,
            selected,
            child,
            footer: None,
        }
    }

    pub fn with_footer(mut self, footer: Line<'static>) -> Self {
        self.footer = Some(footer);
        self
    }
}

impl<W: Widget> Widget for PanelBlock<W> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut block = Block::bordered()
            .title(self.region.as_str())
            .style(Style::default().fg(self.region.color(self.selected)))
            .border_set(border::THICK);
        if let Some(footer) = &self.footer {
            block = block.title_bottom(footer.clone());
        }
        let inner = block.inner(area);
        block.render(area, buf);
        self.child.render(inner, buf);
    }
}
