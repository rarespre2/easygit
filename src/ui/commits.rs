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
    pub hovered: Option<usize>,
}

impl CommitsState {
    pub fn refresh(previous_hovered_id: Option<&str>) -> Self {
        match git::fetch_commits() {
            Ok(commits) => {
                let hovered = preferred_hover_index(&commits, previous_hovered_id);
                Self {
                    commits,
                    status: None,
                    hovered,
                }
            }
            Err(err) => Self {
                commits: Vec::new(),
                status: Some(err),
                hovered: None,
            },
        }
    }

    pub fn move_hover_up(&mut self) {
        self.update_hover(|idx, len| (idx + len - 1) % len);
    }

    pub fn move_hover_down(&mut self) {
        self.update_hover(|idx, len| (idx + 1) % len);
    }

    fn update_hover<F: FnOnce(usize, usize) -> usize>(&mut self, next: F) {
        let len = self.commits.len();
        if len == 0 {
            self.hovered = None;
            return;
        }
        self.hovered = Some(match self.hovered {
            Some(idx) => next(idx, len),
            None => 0,
        });
    }

    pub fn hovered_commit_id(&self) -> Option<&str> {
        self.hovered
            .and_then(|idx| self.commits.get(idx))
            .map(|c| c.id.as_str())
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
            .enumerate()
            .map(|(idx, commit)| {
                let branch_label = format_branch_label(&commit.branches);
                let padded = pad_branch(&branch_label, 14);
                let is_hovered = Some(idx) == self.state.hovered;
                let mut style = Style::default();
                if is_hovered {
                    style = style.fg(Color::Black).bg(Color::Cyan);
                }
                let line = Line::from(vec![
                    Span::styled(padded, Style::default().fg(Color::Cyan)),
                    Span::raw(" "),
                    Span::styled(commit.id.clone(), style),
                    Span::raw(" "),
                    Span::styled(commit.summary.clone(), style),
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
    let truncated = if label.len() > width {
        let mut s = label
            .chars()
            .take(width.saturating_sub(1))
            .collect::<String>();
        s.push('…');
        s
    } else {
        label.to_string()
    };
    format!("{truncated:<width$}")
}

fn preferred_hover_index(commits: &[Commit], previous_id: Option<&str>) -> Option<usize> {
    if commits.is_empty() {
        return None;
    }
    if let Some(id) = previous_id {
        if let Some(idx) = commits.iter().position(|c| c.id == id) {
            return Some(idx);
        }
    }
    Some(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_commit(id: &str, summary: &str, branches: &[&str]) -> Commit {
        Commit {
            id: id.to_string(),
            summary: summary.to_string(),
            branches: branches.iter().map(|b| b.to_string()).collect(),
        }
    }

    #[test]
    fn preferred_hover_keeps_previous_when_present() {
        let commits = vec![
            make_commit("a1b2", "first", &["main"]),
            make_commit("c3d4", "second", &["feature"]),
        ];

        let hovered = preferred_hover_index(&commits, Some("c3d4"));

        assert_eq!(hovered, Some(1));
    }

    #[test]
    fn preferred_hover_defaults_to_first_when_missing() {
        let commits = vec![make_commit("a1b2", "first", &["main"])];

        let hovered = preferred_hover_index(&commits, Some("ffff"));

        assert_eq!(hovered, Some(0));
    }

    #[test]
    fn format_branch_label_handles_various_cases() {
        assert_eq!(format_branch_label(&[]), "-");
        assert_eq!(format_branch_label(&["main".into()]), "main");
        assert_eq!(
            format_branch_label(&["feature".into(), "bugfix".into()]),
            "feature,bugfix"
        );
    }

    #[test]
    fn pad_branch_truncates_and_pads() {
        assert_eq!(pad_branch("short", 10), "short     ");
        assert_eq!(pad_branch("averylongbranch", 8), "averylo…");
    }

    #[test]
    fn hover_moves_wrap() {
        let commits = vec![
            make_commit("a1", "first", &["main"]),
            make_commit("b2", "second", &["feature"]),
        ];
        let mut state = CommitsState {
            commits,
            status: None,
            hovered: None,
        };

        state.move_hover_down();
        assert_eq!(state.hovered, Some(0));

        state.move_hover_down();
        assert_eq!(state.hovered, Some(1));

        state.move_hover_down();
        assert_eq!(state.hovered, Some(0));

        state.move_hover_up();
        assert_eq!(state.hovered, Some(1));
    }
}
