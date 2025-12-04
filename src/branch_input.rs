use crossterm::event::KeyCode;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

use crate::ui::layout::centered_rect;

#[derive(Debug, Default)]
pub struct BranchInput {
    pub value: String,
    pub error: Option<String>,
    pub cursor: usize,
}

impl BranchInput {
    pub fn clamp_cursor(&mut self) {
        if self.cursor > self.value.len() {
            self.cursor = self.value.len();
        }
    }

    pub fn handle_edit_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Backspace => self.remove_prev(),
            KeyCode::Delete => self.remove_next(),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Char(' ') | KeyCode::Char('-') => self.insert_dash(),
            KeyCode::Char(c) => self.insert_char(c),
            _ => {}
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        self.error = None;
    }

    fn insert_dash(&mut self) {
        if self.can_insert_dash() {
            self.insert_char('-');
        }
    }

    fn can_insert_dash(&self) -> bool {
        let prev_is_dash = self
            .value
            .get(..self.cursor)
            .and_then(|s| s.chars().last())
            == Some('-');
        let next_is_dash = self
            .value
            .get(self.cursor..)
            .and_then(|s| s.chars().next())
            == Some('-');
        !prev_is_dash && !next_is_dash
    }

    fn move_left(&mut self) {
        if self.cursor == 0 {
            return;
        }
        if let Some(prev) = self.value[..self.cursor].chars().last() {
            self.cursor -= prev.len_utf8();
        } else {
            self.cursor = 0;
        }
    }

    fn move_right(&mut self) {
        if self.cursor >= self.value.len() {
            return;
        }
        if let Some(next) = self.value[self.cursor..].chars().next() {
            self.cursor += next.len_utf8();
        } else {
            self.cursor = self.value.len();
        }
    }

    fn remove_prev(&mut self) {
        if self.cursor == 0 {
            return;
        }
        if let Some(prev) = self.value[..self.cursor].chars().last() {
            let start = self.cursor - prev.len_utf8();
            self.value.drain(start..self.cursor);
            self.cursor = start;
            self.error = None;
        }
    }

    fn remove_next(&mut self) {
        if self.cursor >= self.value.len() {
            return;
        }
        if let Some(next) = self.value[self.cursor..].chars().next() {
            let end = self.cursor + next.len_utf8();
            self.value.drain(self.cursor..end);
            self.error = None;
        }
    }
}

pub fn render_branch_popup(area: Rect, buf: &mut ratatui::buffer::Buffer, input: &BranchInput) {
    let popup_area = centered_rect(40, 10, area);

    Clear.render(popup_area, buf);

    let mut lines = vec![
        Line::from("New branch name:"),
        Line::from(render_input_line(input)),
    ];

    if let Some(err) = &input.error {
        lines.push(Line::from(err.as_str()).style(Style::default().fg(Color::Red)));
    }

    Paragraph::new(lines)
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title(
                    Line::from(Span::styled(
                        "Create Branch",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )),
                )
                .title_bottom(Line::from(vec![
                    Span::styled(
                        "[Enter] Create",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("   "),
                    Span::styled(
                        "[Esc] Cancel",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]))
                .borders(Borders::ALL)
                .border_set(border::THICK)
                .style(Style::default().fg(Color::Green)),
        )
        .render(popup_area, buf);
}

fn render_input_line(input: &BranchInput) -> Vec<Span<'_>> {
    let cursor = input.cursor.min(input.value.len());
    let mut spans = vec![Span::raw("> ")];

    let left = &input.value[..cursor];
    spans.push(Span::raw(left));

    if cursor < input.value.len() {
        let mut chars = input.value[cursor..].chars();
        if let Some(ch) = chars.next() {
            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(Color::Black).bg(Color::Cyan),
            ));
        }
        let remainder: String = chars.collect();
        if !remainder.is_empty() {
            spans.push(Span::raw(remainder));
        }
    } else {
        spans.push(Span::styled("█", Style::default().fg(Color::Cyan)));
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spaces_and_dashes_insert_single_dash() {
        let mut input = BranchInput::default();
        input.handle_edit_key(KeyCode::Char('f'));
        input.handle_edit_key(KeyCode::Char(' '));
        input.handle_edit_key(KeyCode::Char('-'));

        assert_eq!(input.value, "f-");
        assert_eq!(input.cursor, 2);
    }

    #[test]
    fn prevents_consecutive_dashes() {
        let mut input = BranchInput::default();
        input.handle_edit_key(KeyCode::Char('f'));
        input.handle_edit_key(KeyCode::Char('-'));
        input.handle_edit_key(KeyCode::Char(' '));
        input.handle_edit_key(KeyCode::Char('-'));

        assert_eq!(input.value, "f-");
        assert_eq!(input.cursor, 2);
    }
}
