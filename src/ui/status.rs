use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::{
    git::{ChangeType, FileChange, RepoStatus},
    regions::Region,
};

pub struct StatusBox<'a> {
    status: &'a RepoStatus,
    region: Region,
}

impl<'a> StatusBox<'a> {
    pub fn new(status: &'a RepoStatus, region: Region) -> Self {
        Self { status, region }
    }
}

impl Widget for StatusBox<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = self
            .status
            .repo_name
            .as_deref()
            .map(|name| format!("Workspace • {name}"))
            .unwrap_or_else(|| "Workspace".to_string());
        let mut block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default().fg(Color::Yellow));
        block = block
            .title_bottom(keys_hint_line(self.region))
            .border_set(ratatui::symbols::border::THICK);
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

    vec![overview_line(status)]
}

fn overview_line(status: &RepoStatus) -> Line<'static> {
    let mut spans = Vec::new();

    spans.push(clean_badge(status));
    spans.push(Span::raw("  "));

    let summary_style = Style::default().fg(Color::Yellow);
    spans.push(Span::styled(summary_text(status), summary_style));

    Line::from(spans)
}

fn keys_hint_line(region: Region) -> Line<'static> {
    let mut text = vec!["[q] quit".to_string(), "[l] local changes".to_string()];
    let specific = region.instructions();
    if !specific.is_empty() {
        text.push("│".to_string());
        text.extend(specific.into_iter().map(|s| s.to_string()));
    }

    let hint = text.join("  ");
    Line::from(Span::styled(hint, Style::default().fg(Color::Yellow)))
}

fn clean_badge(status: &RepoStatus) -> Span<'static> {
    if status.is_clean() {
        Span::styled("✓ clean", Style::default().fg(Color::Green))
    } else {
        Span::styled("● dirty", Style::default().fg(Color::Red))
    }
}

fn summary_text(status: &RepoStatus) -> String {
    let parts = summarize_change_counts(&status.changes)
        .into_iter()
        .filter(|(_, count)| *count > 0)
        .map(|(change, count)| format!("{count} {}", change_label(change)))
        .collect::<Vec<_>>();

    if parts.is_empty() {
        format!("{} changes", status.total_changes())
    } else {
        format!("{} changes ({})", status.total_changes(), parts.join(", "))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_counts_and_order() {
        let changes = vec![
            FileChange {
                path: "a".into(),
                change: ChangeType::Added,
                staged: false,
            },
            FileChange {
                path: "b".into(),
                change: ChangeType::Added,
                staged: false,
            },
            FileChange {
                path: "c".into(),
                change: ChangeType::Renamed,
                staged: false,
            },
            FileChange {
                path: "d".into(),
                change: ChangeType::Unknown,
                staged: false,
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
                    staged: false,
                },
                FileChange {
                    path: "b".into(),
                    change: ChangeType::Untracked,
                    staged: false,
                },
            ],
            ..RepoStatus::default()
        };

        assert_eq!(
            summary_text(&status),
            "2 changes (1 added, 1 untracked)".to_string()
        );
    }

    #[test]
    fn shows_branch_hints_without_footer_keys() {
        let line = keys_hint_line(Region::Branches);
        let content = line
            .spans
            .iter()
            .map(|s| s.content.clone())
            .collect::<Vec<_>>()
            .join("");
        assert!(content.contains("[q] quit"));
        assert!(content.contains("[l] local changes"));
        assert!(content.contains("[↑↓] move"));
        assert!(content.contains("[Enter] checkout"));
        assert!(content.contains("[u] update"));
        assert!(content.contains("[p] push"));
        assert!(content.contains("[a] add"));
        assert!(content.contains("[x] delete"));
    }

    #[test]
    fn shows_simple_hints_for_commits() {
        let line = keys_hint_line(Region::Commits);
        let content = line
            .spans
            .iter()
            .map(|s| s.content.clone())
            .collect::<Vec<_>>()
            .join("");
        assert!(content.contains("[q] quit"));
        assert!(content.contains("[↑↓] move"));
    }

    #[test]
    fn keys_footer_can_inherit_border_color() {
        let colored = keys_hint_line(Region::Branches).fg(Color::Yellow);
        assert_eq!(colored.style.fg, Some(Color::Yellow));
    }
}
