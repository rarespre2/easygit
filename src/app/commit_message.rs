use crossterm::event::KeyCode;

use crate::{App, git, ui::input::TextInput};

impl App {
    pub fn handle_commit_message_key(&mut self, code: KeyCode) {
        if !self.commit_message_editing {
            return;
        }

        match code {
            KeyCode::Enter => {
                if self.commit_input.value.trim().is_empty() {
                    self.show_notification("Commit message cannot be empty".to_string());
                    return;
                }
                match git::commit_staged(&self.commit_input.value) {
                    Ok(()) => {
                        let summary = self.commit_input.value.clone();
                        self.commit_input = TextInput::default();
                        self.refresh_status();
                        self.refresh_commits();
                        self.show_notification(format!("Committed: {summary}"));
                        self.commit_message_editing = false;
                    }
                    Err(err) => self.show_notification(err),
                }
            }
            KeyCode::Backspace => {
                self.commit_input.handle_key(KeyCode::Backspace);
            }
            KeyCode::Char(c) if !c.is_control() => {
                self.commit_input.handle_key(KeyCode::Char(c));
            }
            KeyCode::Delete | KeyCode::Left | KeyCode::Right | KeyCode::Home | KeyCode::End => {
                self.commit_input.handle_key(code);
            }
            _ => {}
        }
    }
}
