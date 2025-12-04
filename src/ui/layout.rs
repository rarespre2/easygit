use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn centers_rect_by_percentage() {
        let area = Rect::new(0, 0, 100, 100);
        let centered = centered_rect(40, 20, area);

        assert_eq!(centered.x, 30);
        assert_eq!(centered.y, 40);
        assert_eq!(centered.width, 40);
        assert_eq!(centered.height, 20);
    }
}
