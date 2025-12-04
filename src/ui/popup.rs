use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Widget,
    style::{Color, Style},
    symbols::border,
    widgets::{Block, Borders, Clear},
};

use crate::ui::layout::centered_rect;

pub struct CompartmentPopup;

impl CompartmentPopup {
    pub fn render(area: Rect, buf: &mut Buffer, focus: crate::regions::Region) {
        let popup_area = centered_rect(80, 80, area);

        Clear.render(popup_area, buf);

        let mut frame = Block::default()
            .title("Local changes")
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .style(Style::default().fg(Color::Green));
        frame = frame.title_bottom("[q] close  [↑↓] navigate  [Enter] view");

        let inner = frame.inner(popup_area);
        frame.render(popup_area, buf);

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(inner);

        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(85), Constraint::Percentage(15)])
            .split(columns[1]);

        render_slot(
            columns[0],
            buf,
            crate::regions::Region::Changes,
            matches!(focus, crate::regions::Region::Changes),
        );
        render_slot(
            right[0],
            buf,
            crate::regions::Region::ChangeViewer,
            matches!(focus, crate::regions::Region::ChangeViewer),
        );
        render_slot(
            right[1],
            buf,
            crate::regions::Region::CommitMessage,
            matches!(focus, crate::regions::Region::CommitMessage),
        );
    }
}

fn render_slot(area: Rect, buf: &mut Buffer, region: crate::regions::Region, focused: bool) {
    Block::default()
        .title(region.as_str())
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .style(Style::default().fg(if focused { Color::Green } else { Color::Yellow }))
        .render(area, buf);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_matches_requested_split() {
        let outer = Rect::new(0, 0, 100, 100);
        let popup = centered_rect(80, 80, outer);
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(popup);
        assert_eq!(columns[0].width + columns[1].width, popup.width);
        assert_eq!(columns[0].width, popup.width * 30 / 100);

        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(85), Constraint::Percentage(15)])
            .split(columns[1]);
        assert_eq!(right[0].height + right[1].height, columns[1].height);
        assert_eq!(right[0].height, columns[1].height * 85 / 100);
    }

    #[test]
    fn focus_highlights_selected_slot() {
        let outer = Rect::new(0, 0, 100, 100);
        let mut buf = Buffer::empty(outer);
        CompartmentPopup::render(outer, &mut buf, crate::regions::Region::Changes);
        // Top-left corner border of the left slot should use green when focused.
        assert_eq!(buf[(20, 10)].fg, Color::Green);
    }
}
