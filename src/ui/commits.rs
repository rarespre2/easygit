use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Widget},
};

use crate::git::{self, Commit};
use crate::regions::Region;

use super::panel::PanelBlock;

pub type CommitsPanel<W = super::panel::Empty> = PanelBlock<W>;

pub fn panel(selected: bool) -> CommitsPanel {
    PanelBlock::new(Region::Commits, selected)
}

pub fn panel_with_child<W: Widget>(selected: bool, child: W) -> CommitsPanel<W> {
    PanelBlock::with_child(Region::Commits, selected, child)
}

#[derive(Debug, Default)]
pub struct CommitsState {
    pub commits: Vec<Commit>,
    pub status: Option<String>,
}

impl CommitsState {
    pub fn refresh() -> Self {
        match git::fetch_commits() {
            Ok(commits) => Self {
                commits,
                status: None,
            },
            Err(err) => Self {
                commits: Vec::new(),
                status: Some(err),
            },
        }
    }
}

pub struct CommitList<'a> {
    state: &'a CommitsState,
}

impl<'a> CommitList<'a> {
    pub fn new(state: &'a CommitsState) -> Self {
        Self { state }
    }
}

impl Widget for CommitList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Length(if self.state.status.is_some() { 1 } else { 0 }),
                Constraint::Min(0),
            ])
            .split(area);

        if let Some(status) = self.state.status.as_deref() {
            Paragraph::new(status)
                .style(Style::default().fg(Color::Red))
                .render(chunks[0], buf);
        }

        let list_area = if self.state.status.is_some() {
            chunks[1]
        } else {
            area
        };

        if self.state.commits.is_empty() {
            Paragraph::new("No commits found").render(list_area, buf);
            return;
        }

        let items: Vec<ListItem> = self
            .state
            .commits
            .iter()
            .map(|commit| {
                let branch_label = format_branch_label(&commit.branches);
                let padded = pad_branch(&branch_label, 14);
                let line = Line::from(vec![
                    Span::styled(padded, Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::raw(commit.id.clone()),
                    Span::raw(" "),
                    Span::raw(commit.summary.clone()),
                ]);
                ListItem::new(line)
            })
            .collect();

        List::new(items).render(list_area, buf);
    }
}

fn format_branch_label(branches: &[String]) -> String {
    if branches.is_empty() {
        "-".to_string()
    } else if branches.len() == 1 {
        branches[0].clone()
    } else {
        branches.join(",")
    }
}

fn pad_branch(label: &str, width: usize) -> String {
    let mut out = String::new();
    let mut len = 0;
    for ch in label.chars() {
        if len + 1 > width.saturating_sub(1) {
            out.push('…');
            return format!("{out:<width$}");
        }
        out.push(ch);
        len += 1;
    }
    format!("{out:<width$}")
}
