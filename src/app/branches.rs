use crossterm::event::{KeyCode, KeyEvent};

use crate::{App, branch_input::BranchInput, git, regions::Region, ui::branches};

impl App {
    pub fn refresh_branches(&mut self) {
        let previous = std::mem::take(&mut self.selected_branch);
        self.selected_branch = branches::refresh(previous);
        self.refresh_commits();
    }

    pub fn start_branch_input(&mut self) {
        self.branch_input = Some(BranchInput::default());
    }

    pub fn handle_branch_input_key(&mut self, key_event: KeyEvent) {
        if let Some(input) = self.branch_input.as_mut() {
            input.clamp_cursor();
            match key_event.code {
                KeyCode::Esc => self.branch_input = None,
                KeyCode::Enter => self.submit_branch_input(),
                code => input.handle_edit_key(code),
            }
        }
    }

    pub fn handle_branch_region_keys(&mut self, code: KeyCode) {
        if self.selected_region != Region::Branches {
            return;
        }

        if let Some(message) = match code {
            KeyCode::Char('a') => {
                self.start_branch_input();
                None
            }
            KeyCode::Up
            | KeyCode::Down
            | KeyCode::Enter
            | KeyCode::Delete
            | KeyCode::Char('x')
            | KeyCode::Char('u')
            | KeyCode::Char('p') => branches::handle_key(&mut self.selected_branch, code),
            _ => None,
        } {
            self.show_notification(message);
        }

        self.refresh_commits();
    }

    pub fn submit_branch_input(&mut self) {
        let Some(input) = self.branch_input.as_mut() else {
            return;
        };

        let name = input.value.trim();
        if name.is_empty() {
            input.error = Some("Branch name cannot be empty".to_string());
            return;
        }

        match git::create_branch(name) {
            Ok(()) => {
                let mut previous = std::mem::take(&mut self.selected_branch);
                previous.selected = Some(name.to_string());
                previous.current = Some(name.to_string());
                self.selected_branch = branches::refresh(previous);
                self.branch_input = None;
            }
            Err(err) => {
                input.error = Some(err);
            }
        }
    }
}
