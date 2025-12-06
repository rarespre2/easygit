use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::Widget,
    style::{Color, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::git::{ChangeType, FileChange, RepoStatus};
use crate::ui::layout::centered_rect;
pub struct CompartmentPopup;

impl CompartmentPopup {
    pub fn render(
        area: Rect,
        buf: &mut Buffer,
        focus: crate::regions::Region,
        status: &RepoStatus,
        selected_change: Option<usize>,
        commit_input: &crate::ui::input::TextInput,
        commit_message_editing: bool,
    ) {
        let popup_area = centered_rect(80, 80, area);

        dim_background(area, popup_area, buf);
        Clear.render(popup_area, buf);

        let frame = Block::default()
            .title(keys_hint_line(focus))
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .style(Style::default().fg(Color::Green));

        let inner = frame.inner(popup_area);
        frame.render(popup_area, buf);

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(inner);

        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(85), Constraint::Percentage(15)])
            .split(columns[1]);

        render_slot(
            columns[0],
            buf,
            crate::regions::Region::Changes,
            matches!(focus, crate::regions::Region::Changes),
            status,
            selected_change,
            commit_input,
            commit_message_editing,
        );
        render_slot(
            right[0],
            buf,
            crate::regions::Region::ChangeViewer,
            matches!(focus, crate::regions::Region::ChangeViewer),
            status,
            selected_change,
            commit_input,
            commit_message_editing,
        );
        render_slot(
            right[1],
            buf,
            crate::regions::Region::CommitMessage,
            matches!(focus, crate::regions::Region::CommitMessage),
            status,
            None,
            commit_input,
            commit_message_editing,
        );
    }
}

fn dim_background(area: Rect, popup_area: Rect, buf: &mut Buffer) {
    let overlay = Style::default()
        .bg(Color::Rgb(30, 30, 34))
        .fg(Color::Rgb(30, 30, 34));

    let x_end = area.x.saturating_add(area.width);
    let y_end = area.y.saturating_add(area.height);

    let popup_x_end = popup_area.x.saturating_add(popup_area.width);
    let popup_y_end = popup_area.y.saturating_add(popup_area.height);

    for y in area.y..y_end {
        for x in area.x..x_end {
            let inside_popup = (popup_area.x..popup_x_end).contains(&x)
                && (popup_area.y..popup_y_end).contains(&y);
            if inside_popup {
                continue;
            }
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_symbol(" ");
                cell.set_style(overlay);
            }
        }
    }
}

fn keys_hint_line(region: crate::regions::Region) -> String {
    let mut parts = vec!["[q] close".to_string()];
    let instructions = region.instructions();
    if !instructions.is_empty() {
        parts.push("|".to_string());
        parts.extend(instructions.into_iter().map(|s| s.to_string()));
    }
    format!("Local changes  ·  {}", parts.join("  "))
}

fn render_slot(
    area: Rect,
    buf: &mut Buffer,
    region: crate::regions::Region,
    focused: bool,
    status: &RepoStatus,
    selected_change: Option<usize>,
    commit_input: &crate::ui::input::TextInput,
    commit_message_editing: bool,
) {
    let title = match region {
        crate::regions::Region::Changes => Line::from(vec![
            Span::raw(region.as_str()),
            Span::raw("  ·  "),
            Span::styled("staged", Style::default().fg(Color::Green)),
            Span::raw(" | "),
            Span::styled("unstaged", Style::default().fg(Color::Red)),
        ]),
        _ => Line::from(region.as_str()),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_set(border::THICK)
        .style(Style::default().fg(if focused { Color::Green } else { Color::Yellow }));
    let inner = block.inner(area);
    block.render(area, buf);

    match region {
        crate::regions::Region::Changes => render_changes(inner, buf, status, selected_change),
        crate::regions::Region::ChangeViewer => {
            render_change_viewer(inner, buf, status, selected_change)
        }
        crate::regions::Region::CommitMessage => {
            render_commit_message(inner, buf, commit_input, focused, commit_message_editing)
        }
        _ => {}
    }
}

fn render_changes(
    area: Rect,
    buf: &mut Buffer,
    status: &RepoStatus,
    selected_change: Option<usize>,
) {
    if status.changes.is_empty() {
        Paragraph::new("No changes").render(area, buf);
        return;
    }

    let highlight = Style::default().bg(Color::DarkGray);
    let (start, end) = viewport(status.changes.len(), selected_change, area.height);
    let lines: Vec<Line> = status.changes[start..end]
        .iter()
        .enumerate()
        .map(|(offset, change)| {
            let idx = start + offset;
            let mut line = change_line(change);
            if selected_change == Some(idx) {
                for span in line.spans.iter_mut() {
                    span.style = span.style.patch(highlight);
                }
            }
            line
        })
        .collect();
    Paragraph::new(lines).render(area, buf);
}

fn render_change_viewer(
    area: Rect,
    buf: &mut Buffer,
    status: &RepoStatus,
    selected_change: Option<usize>,
) {
    if let Some(change) = selected_change.and_then(|idx| status.changes.get(idx)) {
        let message = format!("Change view for {} is not yet implemented", change.path);
        Paragraph::new(message).render(area, buf);
    } else {
        Paragraph::new("Select a change to view").render(area, buf);
    }
}

fn render_commit_message(
    area: Rect,
    buf: &mut Buffer,
    input: &crate::ui::input::TextInput,
    focused: bool,
    editing: bool,
) {
    let mode = if editing { "INSERT" } else { "NAV" };
    let mut spans = vec![Span::raw(format!("[{mode}] Commit message "))];
    spans.extend(input.render_line("> "));
    Paragraph::new(Line::from(spans)).render(area, buf);
}

fn change_line(change: &FileChange) -> Line<'static> {
    let (label, color) = if change.staged {
        ("staged", Color::Green)
    } else {
        ("unstaged", Color::Red)
    };

    Line::from(vec![
        Span::styled(label, Style::default().fg(color)),
        Span::raw(" "),
        Span::styled(change.path.clone(), Style::default().fg(color)),
    ])
}

fn viewport(len: usize, focused: Option<usize>, height: u16) -> (usize, usize) {
    if len == 0 || height == 0 {
        return (0, 0);
    }
    let visible = height as usize;
    let focus = focused.unwrap_or(0).min(len.saturating_sub(1));
    if len <= visible {
        return (0, len);
    }
    let max_start = len - visible;
    let start = focus.saturating_sub(visible / 2).min(max_start);
    let end = start + visible;
    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_matches_requested_split() {
        let outer = Rect::new(0, 0, 100, 100);
        let popup = centered_rect(80, 80, outer);
        let frame = Block::default()
            .borders(Borders::ALL)
            .border_set(border::THICK);
        let inner = frame.inner(popup);

        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(inner);
        assert_eq!(columns[0].width + columns[1].width, inner.width);
        assert_eq!(columns[0].width, inner.width * 30 / 100);

        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(85), Constraint::Percentage(15)])
            .split(columns[1]);
        assert_eq!(right[0].height + right[1].height, columns[1].height);
        let expected = columns[1].height * 85 / 100;
        assert!(
            (right[0].height as i32 - expected as i32).abs() <= 1,
            "expected right[0] height near {expected}, got {}",
            right[0].height
        );
    }

    #[test]
    fn focus_highlights_selected_slot() {
        let outer = Rect::new(0, 0, 100, 100);
        let mut buf = Buffer::empty(outer);
        let status = RepoStatus::default();
        let input = crate::ui::input::TextInput::default();
        CompartmentPopup::render(
            outer,
            &mut buf,
            crate::regions::Region::Changes,
            &status,
            None,
            &input,
            false,
        );

        let popup = centered_rect(80, 80, outer);
        let frame = Block::default()
            .borders(Borders::ALL)
            .border_set(border::THICK);
        let inner = frame.inner(popup);
        let left_slot = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(inner)[0];

        // Top-left corner border of the left slot should use green when focused.
        assert_eq!(buf[(left_slot.x, left_slot.y)].fg, Color::Green);
    }

    #[test]
    fn dims_background_outside_popup() {
        let outer = Rect::new(0, 0, 50, 20);
        let mut buf = Buffer::empty(outer);
        let status = RepoStatus::default();
        let input = crate::ui::input::TextInput::default();
        CompartmentPopup::render(
            outer,
            &mut buf,
            crate::regions::Region::Changes,
            &status,
            None,
            &input,
            false,
        );

        let popup = centered_rect(80, 80, outer);
        let outside = &buf[(0, 0)];
        let inside = &buf[(popup.x + 1, popup.y + 1)];

        assert_eq!(outside.bg, Color::Rgb(30, 30, 34));
        assert_eq!(outside.fg, Color::Rgb(30, 30, 34));

        assert_ne!(inside.bg, Color::Black);
        assert_ne!(inside.fg, Color::DarkGray);
    }

    #[test]
    fn change_line_colors_by_stage_status() {
        let staged = FileChange {
            path: "tracked.rs".into(),
            change: ChangeType::Modified,
            staged: true,
        };
        let unstaged = FileChange {
            path: "file.rs".into(),
            change: ChangeType::Modified,
            staged: false,
        };
        let untracked = FileChange {
            path: "new.rs".into(),
            change: ChangeType::Untracked,
            staged: false,
        };

        let staged_line = change_line(&staged);
        let unstaged_line = change_line(&unstaged);
        let untracked_line = change_line(&untracked);

        assert_eq!(staged_line.spans[0].style.fg, Some(Color::Green));
        assert_eq!(unstaged_line.spans[0].style.fg, Some(Color::Red));
        assert_eq!(untracked_line.spans[0].style.fg, Some(Color::Red));
    }

    #[test]
    fn highlights_selected_change_line() {
        let outer = Rect::new(0, 0, 80, 20);
        let mut buf = Buffer::empty(outer);
        let status = RepoStatus {
            changes: vec![
                FileChange {
                    path: "first.rs".into(),
                    change: ChangeType::Modified,
                    staged: false,
                },
                FileChange {
                    path: "second.rs".into(),
                    change: ChangeType::Added,
                    staged: true,
                },
            ],
            ..RepoStatus::default()
        };
        let input = crate::ui::input::TextInput::default();

        CompartmentPopup::render(
            outer,
            &mut buf,
            crate::regions::Region::Changes,
            &status,
            Some(1),
            &input,
            false,
        );

        let popup = centered_rect(80, 80, outer);
        let frame = Block::default()
            .borders(Borders::ALL)
            .border_set(border::THICK);
        let inner = frame.inner(popup);
        let left = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(inner)[0];
        let content = Block::default()
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .inner(left);

        let first_cell = &buf[(content.x, content.y)];
        let second_cell = &buf[(content.x, content.y + 1)];

        assert_ne!(first_cell.bg, Color::DarkGray);
        assert_eq!(second_cell.bg, Color::DarkGray);
    }

    #[test]
    fn change_viewer_shows_placeholder_with_selected_file() {
        let outer = Rect::new(0, 0, 80, 20);
        let mut buf = Buffer::empty(outer);
        let status = RepoStatus {
            changes: vec![FileChange {
                path: "dir/file.rs".into(),
                change: ChangeType::Modified,
                staged: false,
            }],
            ..RepoStatus::default()
        };
        let input = crate::ui::input::TextInput::default();

        CompartmentPopup::render(
            outer,
            &mut buf,
            crate::regions::Region::ChangeViewer,
            &status,
            Some(0),
            &input,
            false,
        );

        let popup = centered_rect(80, 80, outer);
        let frame = Block::default()
            .borders(Borders::ALL)
            .border_set(border::THICK);
        let inner = frame.inner(popup);
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(inner);
        let change_viewer_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(85), Constraint::Percentage(15)])
            .split(columns[1])[0];
        let change_viewer_inner = Block::default()
            .borders(Borders::ALL)
            .border_set(border::THICK)
            .inner(change_viewer_area);

        let mut text = String::new();
        for y in change_viewer_inner.y..change_viewer_inner.y + change_viewer_inner.height {
            for x in change_viewer_inner.x..change_viewer_inner.x + change_viewer_inner.width {
                text.push_str(buf[(x, y)].symbol());
            }
        }

        assert!(
            text.contains("Change view for dir/file.rs"),
            "text was: {text}"
        );
    }
}
