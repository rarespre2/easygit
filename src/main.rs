use branch_input::BranchInput;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};
use std::{
    io,
    time::{Duration, Instant},
};

use crate::git::{BranchInfo, RepoStatus};
use crate::regions::Region;
use crate::ui::{branches, commits, details, popup, stashes, status};
use notification::{render_notification, Notification};

mod app;
mod branch_input;
mod git;
mod notification;
mod regions;
mod ui;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug)]
pub struct App {
    selected_region: Region,
    exit: bool,
    selected_branch: BranchInfo,
    commits: commits::CommitsState,
    hovered_commit_id: Option<String>,
    branch_input: Option<BranchInput>,
    repo_status: RepoStatus,
    last_refresh: Instant,
    refresh_interval: Duration,
    notification: Option<Notification>,
    show_changes_popup: bool,
    popup_region: Region,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            selected_region: Region::default(),
            exit: false,
            selected_branch: BranchInfo::default(),
            commits: commits::CommitsState::default(),
            hovered_commit_id: None,
            branch_input: None,
            repo_status: RepoStatus::default(),
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_millis(1000),
            notification: None,
            show_changes_popup: false,
            popup_region: Region::Changes,
        };
        app.refresh_all();
        app
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            self.refresh_if_due();
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn refresh_if_due(&mut self) {
        self.clear_expired_notification();
        if self.last_refresh.elapsed() >= self.refresh_interval {
            self.refresh_all();
            self.last_refresh = Instant::now();
        }
    }

    fn refresh_all(&mut self) {
        self.refresh_branches();
        self.refresh_status();
        self.last_refresh = Instant::now();
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if !event::poll(self.refresh_interval)? {
            return Ok(());
        }

        if let Event::Key(key_event) = event::read()? {
            if should_handle_key(&key_event) {
                self.handle_key_event(key_event);
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.branch_input.is_some() {
            self.handle_branch_input_key(key_event);
            return;
        }

        if self.show_changes_popup {
            self.handle_popup_keys(key_event.code);
            return;
        }

        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('b') => self.select_region(Region::Branches),
            KeyCode::Char('c') => self.select_region(Region::Commits),
            KeyCode::Char('d') => self.select_region(Region::Details),
            KeyCode::Char('s') => self.select_region(Region::Stashes),
            KeyCode::Char('l') => {
                self.show_changes_popup = true;
                self.popup_region = Region::Changes;
            }
            code => {
                self.handle_branch_region_keys(code);
                self.handle_commits_region_keys(code);
            }
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn select_region(&mut self, region: Region) {
        self.selected_region = region;
    }

    fn refresh_status(&mut self) {
        self.repo_status = git::fetch_repo_status();
    }

    fn show_notification(&mut self, message: String) {
        self.notification = Some(Notification {
            message,
            expires_at: Instant::now() + Duration::from_secs(10),
        });
    }

    fn clear_expired_notification(&mut self) {
        if let Some(notification) = &self.notification {
            if Instant::now() >= notification.expires_at {
                self.notification = None;
            }
        }
    }

}

fn should_handle_key(key_event: &KeyEvent) -> bool {
    matches!(
        key_event.kind,
        KeyEventKind::Press | KeyEventKind::Repeat
    )
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(10), Constraint::Percentage(90)])
            .split(area);
        status::StatusBox::new(&self.repo_status, self.selected_region).render(layout[0], buf);

        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(layout[1]);

        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(outer_layout[0]);

        let right_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(outer_layout[1]);

        branches::panel_with_child(
            self.selected_region == Region::Branches,
            branches::BranchList::new(&self.selected_branch),
        )
        .render(left_layout[0], buf);
        stashes::panel(self.selected_region == Region::Stashes).render(left_layout[1], buf);
        commits::panel_with_child(
            self.selected_region == Region::Commits,
            commits::CommitList::new(&self.commits),
        )
        .render(right_layout[0], buf);
        details::panel(self.selected_region == Region::Details).render(right_layout[1], buf);

        if let Some(input) = &self.branch_input {
            branch_input::render_branch_popup(area, buf, input);
        }

        if let Some(notification) = &self.notification {
            render_notification(area, buf, notification);
        }

        if self.show_changes_popup {
            popup::CompartmentPopup::render(area, buf, self.popup_region);
        }
    }
}

impl App {
    fn handle_popup_keys(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') | KeyCode::Esc => self.show_changes_popup = false,
            KeyCode::Char('c') => self.popup_region = Region::Changes,
            KeyCode::Char('v') => self.popup_region = Region::ChangeViewer,
            KeyCode::Char('m') => self.popup_region = Region::CommitMessage,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    #[test]
    fn should_handle_press_and_repeat_keys() {
        let press = KeyEvent::new(KeyCode::Up, KeyModifiers::NONE);
        assert!(should_handle_key(&press));

        let repeat =
            KeyEvent::new_with_kind(KeyCode::Up, KeyModifiers::NONE, KeyEventKind::Repeat);
        assert!(should_handle_key(&repeat));

        let release =
            KeyEvent::new_with_kind(KeyCode::Up, KeyModifiers::NONE, KeyEventKind::Release);
        assert!(!should_handle_key(&release));
    }
}
