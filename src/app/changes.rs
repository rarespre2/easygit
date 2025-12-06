use crossterm::event::KeyCode;

use crate::{App, git, regions::Region};

impl App {
    pub fn handle_changes_popup_key(&mut self, code: KeyCode) {
        if self.popup_region != Region::Changes {
            return;
        }

        match code {
            KeyCode::Up => self.move_change_selection(-1),
            KeyCode::Down => self.move_change_selection(1),
            KeyCode::Enter => self.toggle_stage_selected_change(),
            KeyCode::Char('x') => self.discard_selected_change(),
            _ => {}
        }
    }

    pub fn ensure_change_selection(&mut self) {
        let len = self.repo_status.changes.len();
        if len == 0 {
            self.selected_change = None;
            return;
        }

        let current = self.selected_change.unwrap_or(0).min(len - 1);
        self.selected_change = Some(current);
    }

    fn move_change_selection(&mut self, delta: isize) {
        let len = self.repo_status.changes.len();
        if len == 0 {
            self.selected_change = None;
            return;
        }

        let current = self.selected_change.unwrap_or(0).min(len - 1) as isize;
        let next = (current + delta).clamp(0, len.saturating_sub(1) as isize) as usize;
        self.selected_change = Some(next);
    }

    fn toggle_stage_selected_change(&mut self) {
        let Some(idx) = self.selected_change else {
            return;
        };
        let Some(change) = self.repo_status.changes.get(idx) else {
            return;
        };

        let path = change.path.clone();
        let result = if change.staged {
            git::unstage_change(&path)
        } else {
            git::stage_change(&path)
        };

        if let Err(err) = result {
            self.show_notification(err);
            return;
        }

        self.refresh_status();
        self.reselect_change(Some(path));
    }

    fn discard_selected_change(&mut self) {
        let Some(idx) = self.selected_change else {
            return;
        };
        let Some(change) = self.repo_status.changes.get(idx) else {
            return;
        };

        let path = change.path.clone();
        if let Err(err) = git::discard_change(&path) {
            self.show_notification(err);
            return;
        }

        self.refresh_status();
        self.reselect_change(None);
    }

    pub fn reselect_change(&mut self, preferred_path: Option<String>) {
        let len = self.repo_status.changes.len();
        if len == 0 {
            self.selected_change = None;
            return;
        }

        if let Some(path) = preferred_path {
            if let Some(idx) = self.repo_status.changes.iter().position(|c| c.path == path) {
                self.selected_change = Some(idx);
                return;
            }
        }

        let current = self.selected_change.unwrap_or(0).min(len - 1);
        self.selected_change = Some(current);
    }
}
