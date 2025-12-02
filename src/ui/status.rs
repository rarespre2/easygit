use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::git::{ChangeType, FileChange, RepoStatus};

const MAX_LISTED_CHANGES: usize = 5;

pub struct StatusBox<'a> {
    status: &'a RepoStatus,
}

impl<'a> StatusBox<'a> {
    pub fn new(status: &'a RepoStatus) -> Self {
        Self { status }
    }
}

impl Widget for StatusBox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = format!("Status ({} changes)", self.status.total_changes());
        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default().fg(Color::Yellow));
        let inner = block.inner(area);
        block.render(area, buf);

        let lines = render_lines(self.status);
        Paragraph::new(lines).render(inner, buf);
    }
}

fn render_lines(status: &RepoStatus) -> Vec<Line<'static>> {
    if let Some(err) = &status.error {
        return vec![Line::from(err.clone()).style(Style::default().fg(Color::Red))];
    }

    if status.is_clean() {
        return vec![Line::from("Working tree clean").style(Style::default().fg(Color::Green))];
    }

    let mut lines = Vec::new();
    lines.push(Line::from(summary_text(status)));

    for change in status.changes.iter().take(MAX_LISTED_CHANGES) {
        let label = change_label(change.change);
        let line = Line::from(vec![
            Span::styled(
                format!("- {label}"),
                Style::default().fg(change_color(change.change)),
            ),
            Span::raw(format!(" {}", change.path)),
        ]);
        lines.push(line);
    }

    if status.changes.len() > MAX_LISTED_CHANGES {
        lines.push(Line::from(format!(
            "… and {} more",
            status.changes.len() - MAX_LISTED_CHANGES
        )));
    }

    lines
}

fn summary_text(status: &RepoStatus) -> String {
    let parts = summarize_change_counts(&status.changes)
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(change, count)| format!("{count} {}", change_label(change)))
        .collect::<Vec<_>>();

    if parts.is_empty() {
        format!("Local changes: {}", status.total_changes())
    } else {
        format!(
            "Local changes: {} ({})",
            status.total_changes(),
            parts.join(", ")
        )
    }
}

fn summarize_change_counts(changes: &[FileChange]) -> Vec<(ChangeType, usize)> {
    let mut counts = vec![
        (ChangeType::Added, 0),
        (ChangeType::Modified, 0),
        (ChangeType::Deleted, 0),
        (ChangeType::Renamed, 0),
        (ChangeType::Untracked, 0),
        (ChangeType::TypeChange, 0),
        (ChangeType::Unmerged, 0),
        (ChangeType::Copied, 0),
        (ChangeType::Unknown, 0),
    ];

    for change in changes {
        if let Some((_, count)) = counts.iter_mut().find(|(kind, _)| *kind == change.change) {
            *count += 1;
        }
    }

    counts
}

fn change_label(change: ChangeType) -> &'static str {
    match change {
        ChangeType::Added => "added",
        ChangeType::Modified => "modified",
        ChangeType::Deleted => "deleted",
        ChangeType::Renamed => "renamed",
        ChangeType::Copied => "copied",
        ChangeType::TypeChange => "type-change",
        ChangeType::Untracked => "untracked",
        ChangeType::Unmerged => "unmerged",
        ChangeType::Unknown => "other",
    }
}

fn change_color(change: ChangeType) -> Color {
    match change {
        ChangeType::Added => Color::Green,
        ChangeType::Modified => Color::Yellow,
        ChangeType::Deleted => Color::Red,
        ChangeType::Renamed => Color::Cyan,
        ChangeType::Copied => Color::Blue,
        ChangeType::TypeChange => Color::Blue,
        ChangeType::Untracked => Color::Magenta,
        ChangeType::Unmerged => Color::LightRed,
        ChangeType::Unknown => Color::Gray,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_counts_and_order() {
        let changes = vec![
            FileChange {
                path: "a".into(),
                change: ChangeType::Added,
            },
            FileChange {
                path: "b".into(),
                change: ChangeType::Added,
            },
            FileChange {
                path: "c".into(),
                change: ChangeType::Renamed,
            },
            FileChange {
                path: "d".into(),
                change: ChangeType::Unknown,
            },
        ];

        let counts = summarize_change_counts(&changes);

        assert_eq!(
            counts,
            vec![
                (ChangeType::Added, 2),
                (ChangeType::Modified, 0),
                (ChangeType::Deleted, 0),
                (ChangeType::Renamed, 1),
                (ChangeType::Untracked, 0),
                (ChangeType::TypeChange, 0),
                (ChangeType::Unmerged, 0),
                (ChangeType::Copied, 0),
                (ChangeType::Unknown, 1),
            ]
        );
    }

    #[test]
    fn builds_summary_text() {
        let status = RepoStatus {
            changes: vec![
                FileChange {
                    path: "a".into(),
                    change: ChangeType::Added,
                },
                FileChange {
                    path: "b".into(),
                    change: ChangeType::Untracked,
                },
            ],
            error: None,
        };

        assert_eq!(
            summary_text(&status),
            "Local changes: 2 (1 added, 1 untracked)".to_string()
        );
    }
}
