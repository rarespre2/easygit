use crossterm::event::KeyCode;

use crate::{App, regions::Region, ui::commits};

impl App {
    pub fn refresh_commits(&mut self) {
        self.commits = commits::CommitsState::refresh(self.hovered_commit_id.as_deref());
        self.hovered_commit_id = self.commits.hovered_commit_id().map(|id| id.to_string());
    }

    pub fn handle_commits_region_keys(&mut self, code: KeyCode) {
        if self.selected_region != Region::Commits {
            return;
        }

        match code {
            KeyCode::Up => self.commits.move_hover_up(),
            KeyCode::Down => self.commits.move_hover_down(),
            _ => {}
        }

        self.hovered_commit_id = self.commits.hovered_commit_id().map(|id| id.to_string());
    }
}
