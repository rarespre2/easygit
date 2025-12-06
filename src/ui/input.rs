use crossterm::event::KeyCode;
use ratatui::{
    style::{Color, Style},
    text::Span,
};

#[derive(Debug, Default, Clone)]
pub struct TextInput {
    pub value: String,
    pub cursor: usize,
}

impl TextInput {
    pub fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Backspace => self.remove_prev(),
            KeyCode::Delete => self.remove_next(),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.value.len(),
            KeyCode::Char(c) if !c.is_control() => self.insert_char(c),
            _ => {}
        }
    }

    fn insert_char(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += c.len_utf8();
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
        }
    }

    fn remove_next(&mut self) {
        if self.cursor >= self.value.len() {
            return;
        }
        if let Some(next) = self.value[self.cursor..].chars().next() {
            let end = self.cursor + next.len_utf8();
            self.value.drain(self.cursor..end);
        }
    }

    pub fn render_line<'a>(&'a self, prompt: &'a str) -> Vec<Span<'a>> {
        let cursor = self.cursor.min(self.value.len());
        let mut spans = vec![Span::raw(prompt)];

        let left = &self.value[..cursor];
        spans.push(Span::raw(left));

        if cursor < self.value.len() {
            let mut chars = self.value[cursor..].chars();
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
}
