use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

#[derive(Debug)]
pub struct Notification {
    pub message: String,
    pub expires_at: std::time::Instant,
}

pub fn render_notification(
    area: ratatui::layout::Rect,
    buf: &mut ratatui::buffer::Buffer,
    notification: &Notification,
) {
    if area.width < 10 || area.height < 3 {
        return;
    }

    let message_width = notification.message.chars().count().saturating_add(4);
    let width = message_width.min(area.width as usize) as u16;
    let height = 3;
    let x = area.x + area.width.saturating_sub(width);
    let y = area.y + area.height.saturating_sub(height);
    let popup_area = ratatui::layout::Rect::new(x, y, width, height);

    Clear.render(popup_area, buf);
    Paragraph::new(notification.message.clone())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Notice")
                .style(Style::default().fg(Color::Yellow)),
        )
        .render(popup_area, buf);
}
