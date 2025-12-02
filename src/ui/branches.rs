use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Widget},
};

use crate::git::{self, BranchInfo, BranchSummary};
use crate::regions::Region;

use super::panel::PanelBlock;

pub type BranchesPanel<W = super::panel::Empty> = PanelBlock<W>;

pub fn panel(selected: bool) -> BranchesPanel {
    PanelBlock::new(Region::Branches, selected)
}

pub fn panel_with_child<W: Widget>(selected: bool, child: W) -> BranchesPanel<W> {
    PanelBlock::with_child(Region::Branches, selected, child)
}

pub fn handle_key(info: &mut BranchInfo, key: KeyCode) {
    match key {
        KeyCode::Up => move_hover_up(info),
        KeyCode::Down => move_hover_down(info),
        KeyCode::Enter => checkout_hovered(info),
        KeyCode::Char('x') | KeyCode::Delete => delete_hovered(info),
        _ => {}
    }
}

pub fn refresh(prev: BranchInfo) -> BranchInfo {
    let mut current = git::fetch_branch_info();
    current.hovered = preferred_hover_index(&current, prev.hovered);
    current.selected = prev.selected.filter(|selected| {
        current
            .branches
            .iter()
            .any(|branch| &branch.name == selected)
    });
    current
}

fn move_hover_up(info: &mut BranchInfo) {
    if let Some(hovered) = info.hovered {
        let len = info.branches.len();
        if len > 0 {
            info.hovered = Some((hovered + len - 1) % len);
        }
    } else if !info.branches.is_empty() {
        info.hovered = Some(0);
    }
}

fn move_hover_down(info: &mut BranchInfo) {
    if let Some(hovered) = info.hovered {
        let len = info.branches.len();
        if len > 0 {
            info.hovered = Some((hovered + 1) % len);
        }
    } else if !info.branches.is_empty() {
        info.hovered = Some(0);
    }
}

fn select_hovered(info: &mut BranchInfo) {
    if let Some(index) = info.hovered {
        if let Some(name) = info.branches.get(index) {
            info.current = Some(name.name.clone());
            info.selected = Some(name.name.clone());
        }
    }
}

fn checkout_hovered(info: &mut BranchInfo) {
    if let Some(index) = info.hovered {
        if let Some(branch) = info.branches.get(index).cloned() {
            match git::checkout_branch(&branch.name) {
                Ok(()) => {
                    let previous = std::mem::take(info);
                    let mut refreshed = refresh(previous);
                    refreshed.selected = Some(branch.name.clone());
                    refreshed.current = Some(branch.name);
                    *info = refreshed;
                }
                Err(err) => {
                    info.status = Some(format!("Checkout failed: {err}"));
                }
            }
        }
    }
}

fn delete_hovered(info: &mut BranchInfo) {
    if let Some(index) = info.hovered {
        if let Some(branch) = info.branches.get(index).cloned() {
            if info.current.as_deref() == Some(branch.name.as_str()) {
                info.status = Some("Cannot delete the current branch".to_string());
                return;
            }

            match git::delete_branch(&branch.name) {
                Ok(()) => {
                    let previous = std::mem::take(info);
                    *info = refresh(previous);
                }
                Err(err) => {
                    info.status = Some(format!("Delete failed: {err}"));
                }
            }
        }
    }
}

fn preferred_hover_index(info: &BranchInfo, previous: Option<usize>) -> Option<usize> {
    if info.branches.is_empty() {
        return None;
    }

    if let Some(current_name) = &info.current {
        if let Some(index) = info
            .branches
            .iter()
            .position(|branch| &branch.name == current_name)
        {
            return Some(index);
        }
    }

    Some(
        previous
            .unwrap_or(0)
            .min(info.branches.len().saturating_sub(1)),
    )
}

pub struct BranchList<'a> {
    branches: &'a [BranchSummary],
    current: Option<&'a str>,
    status: Option<&'a str>,
    hovered: Option<usize>,
    selected: Option<&'a str>,
}

impl<'a> BranchList<'a> {
    pub fn new(info: &'a BranchInfo) -> Self {
        Self {
            branches: &info.branches,
            current: info.current.as_deref(),
            status: info.status.as_deref(),
            hovered: info.hovered,
            selected: info.selected.as_deref(),
        }
    }
}

impl Widget for BranchList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.branches.is_empty() {
            if let Some(status) = self.status {
                Paragraph::new(status)
                    .style(Style::default().fg(Color::Red))
                    .render(area, buf);
            } else {
                Paragraph::new("No branches found").render(area, buf);
            }
            return;
        }

        let list_area = if let Some(status) = self.status {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(vec![Constraint::Length(1), Constraint::Min(0)])
                .split(area);
            Paragraph::new(status)
                .style(Style::default().fg(Color::Red))
                .render(chunks[0], buf);
            chunks[1]
        } else {
            area
        };

        let items: Vec<ListItem> = self
            .branches
            .iter()
            .enumerate()
            .map(|(index, branch)| {
                let is_current = Some(branch.name.as_str()) == self.current;
                let is_hovered = Some(index) == self.hovered;
                let is_selected = Some(branch.name.as_str()) == self.selected;
                let prefix = format!(
                    "{}{}",
                    if is_hovered { ">" } else { " " },
                    if is_current { "*" } else { " " }
                );

                let indicator = format_indicator(branch);
                let indicator_len = visible_width(&indicator);
                let width = list_area.width as usize;
                let prefix_len = visible_width(&prefix);
                let available_name = width.saturating_sub(prefix_len + indicator_len + 2).max(0);
                let display_name = truncate_with_ellipsis(&branch.name, available_name);
                let name_len = visible_width(&display_name);
                let padding = " ".repeat(available_name.saturating_sub(name_len));

                let mut spans = vec![
                    Span::raw(prefix),
                    Span::raw(" "),
                    Span::raw(display_name),
                    Span::raw(padding),
                ];
                if indicator_len > 0 && width > prefix_len + 1 {
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(
                        indicator,
                        Style::default().fg(indicator_color(branch)),
                    ));
                }

                let style = if is_current {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default().add_modifier(Modifier::UNDERLINED)
                } else {
                    Style::default()
                }
                .add_modifier(if is_hovered {
                    Modifier::REVERSED
                } else {
                    Modifier::empty()
                });
                ListItem::new(Line::from(spans)).style(style)
            })
            .collect();

        let list = List::new(items);
        list.render(list_area, buf);
    }
}

fn visible_width(text: &str) -> usize {
    text.chars().count()
}

fn truncate_with_ellipsis(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }
    let text_len = text.chars().count();
    if text_len <= max_width {
        return text.to_string();
    }
    if max_width == 1 {
        return "…".to_string();
    }
    let mut truncated = String::new();
    for ch in text.chars().take(max_width - 1) {
        truncated.push(ch);
    }
    truncated.push('…');
    truncated
}

fn format_indicator(branch: &BranchSummary) -> String {
    let ahead = branch.ahead.unwrap_or(0);
    let behind = branch.behind.unwrap_or(0);
    format!("↑{ahead} ↓{behind}")
}

fn indicator_color(branch: &BranchSummary) -> Color {
    let ahead = branch.ahead.unwrap_or(0) > 0;
    let behind = branch.behind.unwrap_or(0) > 0;
    match (ahead, behind) {
        (true, true) => Color::Yellow,
        (true, false) => Color::Green,
        (false, true) => Color::Red,
        _ => Color::Gray,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncates_and_adds_ellipsis() {
        assert_eq!(
            truncate_with_ellipsis("feature/some-long-name", 10),
            "feature/s…"
        );
        assert_eq!(truncate_with_ellipsis("short", 10), "short");
        assert_eq!(truncate_with_ellipsis("long", 1), "…");
    }

    #[test]
    fn formats_indicator_for_ahead_behind() {
        let mut branch = BranchSummary {
            name: "feature".into(),
            ahead: Some(2),
            behind: Some(1),
        };
        assert_eq!(format_indicator(&branch), "↑2 ↓1");
        assert_eq!(indicator_color(&branch), Color::Yellow);

        branch.ahead = Some(0);
        branch.behind = Some(0);
        assert_eq!(format_indicator(&branch), "↑0 ↓0");
        assert_eq!(indicator_color(&branch), Color::Gray);

        branch.ahead = None;
        branch.behind = None;
        assert_eq!(format_indicator(&branch), "↑0 ↓0");
    }
}
