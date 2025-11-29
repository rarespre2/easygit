use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use std::io;

use crate::git::BranchInfo;
use crate::regions::Region;
use crate::ui::{branches, commits, details, stashes};

mod git;
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
    branch_input: Option<BranchInput>,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            selected_region: Region::default(),
            exit: false,
            selected_branch: BranchInfo::default(),
            branch_input: None,
        };
        app.refresh_branches();
        app
    }
}

#[derive(Debug, Default)]
struct BranchInput {
    value: String,
    error: Option<String>,
    cursor: usize,
}

impl BranchInput {
    fn clamp_cursor(&mut self) {
        if self.cursor > self.value.len() {
            self.cursor = self.value.len();
        }
    }

    fn handle_edit_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Backspace => self.remove_prev(),
            KeyCode::Delete => self.remove_next(),
            KeyCode::Left => self.move_left(),
            KeyCode::Right => self.move_right(),
            KeyCode::Char(c) => self.insert_char(c),
            _ => {}
        }
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

    fn insert_char(&mut self, c: char) {
        self.value.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        self.error = None;
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

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
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

        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('b') => self.select_region(Region::Branches),
            KeyCode::Char('c') => self.select_region(Region::Commits),
            KeyCode::Char('d') => self.select_region(Region::Details),
            KeyCode::Char('s') => self.select_region(Region::Stashes),
            code => self.handle_branch_region_keys(code),
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn select_region(&mut self, region: Region) {
        self.selected_region = region;
    }

    fn refresh_branches(&mut self) {
        let previous = std::mem::take(&mut self.selected_branch);
        self.selected_branch = branches::refresh(previous);
    }

    fn start_branch_input(&mut self) {
        self.branch_input = Some(BranchInput::default());
    }

    fn handle_branch_input_key(&mut self, key_event: KeyEvent) {
        if let Some(input) = self.branch_input.as_mut() {
            input.clamp_cursor();
            match key_event.code {
                KeyCode::Esc => self.branch_input = None,
                KeyCode::Enter => self.submit_branch_input(),
                code => input.handle_edit_key(code),
            }
        }
    }

    fn handle_branch_region_keys(&mut self, code: KeyCode) {
        if self.selected_region != Region::Branches {
            return;
        }

        match code {
            KeyCode::Char('a') => self.start_branch_input(),
            KeyCode::Up | KeyCode::Down | KeyCode::Enter | KeyCode::Delete | KeyCode::Char('x') => {
                branches::handle_key(&mut self.selected_branch, code);
            }
            _ => {}
        }
    }

    fn submit_branch_input(&mut self) {
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

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(area);

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
        commits::panel(self.selected_region == Region::Commits).render(right_layout[0], buf);
        details::panel(self.selected_region == Region::Details).render(right_layout[1], buf);

        if let Some(input) = &self.branch_input {
            render_branch_popup(area, buf, input);
        }
    }
}

fn render_branch_popup(area: Rect, buf: &mut Buffer, input: &BranchInput) {
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
        .block(
            Block::default()
                .title("Create branch")
                .title_bottom(
                    Line::from("Enter to create, Esc to cancel")
                        .style(Style::default().fg(Color::Gray)),
                )
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow)),
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

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
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
