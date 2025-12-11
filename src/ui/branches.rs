use std::mem;

use crossterm::event::KeyCode;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{List, ListItem, Paragraph, Widget},
};

use crate::git::{self, BranchInfo, BranchSummary};
use crate::regions::Region;

use super::panel::PanelBlock;

pub type BranchesPanel<W = super::panel::Empty> = PanelBlock<W>;

pub fn panel_with_child<W: Widget>(selected: bool, child: W) -> BranchesPanel<W> {
    PanelBlock::with_child(Region::Branches, selected, child)
}

pub fn panel(selected: bool, info: &BranchInfo) -> BranchPanel<'_> {
    BranchPanel { info, selected }
}

pub struct BranchPanel<'a> {
    info: &'a BranchInfo,
    selected: bool,
}

impl Widget for BranchPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let base_block = ratatui::widgets::Block::bordered()
            .style(Style::default().fg(Region::Branches.color(self.selected)))
            .border_set(ratatui::symbols::border::THICK);
        let inner = base_block.inner(area);

        let title = if let (Some(hovered), true) = (
            self.info.hovered,
            self.info.branches.len() > inner.height as usize,
        ) {
            format!(
                "{} ({}/{})",
                Region::Branches.as_str(),
                hovered + 1,
                self.info.branches.len()
            )
        } else {
            Region::Branches.as_str().to_string()
        };

        let block = base_block.title(title);
        block.render(area, buf);
        BranchList::new(self.info).render(inner, buf);
    }
}

pub fn handle_key(info: &mut BranchInfo, key: KeyCode) -> Option<String> {
    match key {
        KeyCode::Up => {
            move_hover_up(info);
            None
        }
        KeyCode::Down => {
            move_hover_down(info);
            None
        }
        KeyCode::Enter => checkout_hovered(info),
        KeyCode::Char('x') | KeyCode::Delete => delete_hovered(info),
        KeyCode::Char('u') => update_branches(info),
        KeyCode::Char('p') => push_current_branch(info),
        _ => None,
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

fn checkout_hovered(info: &mut BranchInfo) -> Option<String> {
    if let Some(index) = info.hovered {
        if let Some(branch) = info.branches.get(index).cloned() {
            let checkout_result = if branch.is_remote {
                if let Some(remote_ref) = branch.remote_ref.as_ref() {
                    git::checkout_remote_branch(remote_ref)
                } else {
                    return Some("Missing remote reference".to_string());
                }
            } else {
                git::checkout_branch(&branch.name).map(|_| branch.name.clone())
            };

            match checkout_result {
                Ok(_) => {
                    let previous = std::mem::take(info);
                    let mut refreshed = refresh(previous);
                    refreshed.selected = refreshed.current.clone();
                    *info = refreshed;
                    return info
                        .current
                        .as_ref()
                        .map(|name| format!("Switched to {name}"));
                }
                Err(err) => return Some(format!("Checkout failed: {err}")),
            }
        }
    }

    None
}

fn delete_hovered(info: &mut BranchInfo) -> Option<String> {
    if let Some(index) = info.hovered {
        if let Some(branch) = info.branches.get(index).cloned() {
            if branch.is_remote {
                return Some("Cannot delete remote branches".to_string());
            }

            if info.current.as_deref() == Some(branch.name.as_str()) {
                return Some("Cannot delete the current branch".to_string());
            }

            match git::delete_branch(&branch.name) {
                Ok(()) => {
                    let previous = std::mem::take(info);
                    *info = refresh(previous);
                    return Some(format!("Deleted {}", branch.name));
                }
                Err(err) => {
                    return Some(format!("Delete failed: {err}"));
                }
            }
        }
    }

    None
}

fn update_branches(info: &mut BranchInfo) -> Option<String> {
    let fetch_result = git::fetch_remotes();
    let mut previous = mem::take(info);
    previous.status = None;
    *info = refresh(previous);

    if let Err(err) = fetch_result {
        return Some(err);
    }

    if info.current.is_some() {
        pull_current_branch(info)
    } else {
        Some("Fetched remote branches".to_string())
    }
}

fn pull_current_branch(info: &mut BranchInfo) -> Option<String> {
    let Some(current) = info.current.clone() else {
        return Some("No current branch to update".to_string());
    };

    match git::pull_current_branch() {
        Ok(()) => refresh_after_remote_action(info),
        Err(err) => {
            let message = format!("Update {current} failed: {err}");
            Some(message)
        }
    }
}

fn push_current_branch(info: &mut BranchInfo) -> Option<String> {
    let Some(current) = info.current.clone() else {
        return Some("No current branch to push".to_string());
    };

    match git::push_current_branch() {
        Ok(()) => refresh_after_remote_action(info),
        Err(err) => {
            let message = format!("Push {current} failed: {err}");
            Some(message)
        }
    }
}

fn refresh_after_remote_action(info: &mut BranchInfo) -> Option<String> {
    let mut previous = mem::take(info);
    previous.status = None;
    *info = refresh(previous);
    info.current
        .as_ref()
        .map(|branch| format!("Updated {}", branch))
}

fn preferred_hover_index(info: &BranchInfo, previous: Option<usize>) -> Option<usize> {
    if info.branches.is_empty() {
        return None;
    }

    if let Some(previous) = previous {
        return Some(previous.min(info.branches.len().saturating_sub(1)));
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

    Some(0)
}

pub struct BranchList<'a> {
    branches: &'a [BranchSummary],
    current: Option<&'a str>,
    hovered: Option<usize>,
    selected: Option<&'a str>,
}

impl<'a> BranchList<'a> {
    pub fn new(info: &'a BranchInfo) -> Self {
        Self {
            branches: &info.branches,
            current: info.current.as_deref(),
            hovered: info.hovered,
            selected: info.selected.as_deref(),
        }
    }
}

impl Widget for BranchList<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.branches.is_empty() {
            Paragraph::new("No branches found").render(area, buf);
            return;
        }

        let (start, end) = viewport(self.branches.len(), self.hovered, area.height);
        let items: Vec<ListItem> = self.branches[start..end]
            .iter()
            .enumerate()
            .map(|(offset, branch)| {
                let index = start + offset;
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
                let width = area.width as usize;
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
                    spans.push(Span::raw(indicator));
                }

                let base_style = if is_current {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default().add_modifier(Modifier::UNDERLINED)
                } else {
                    Style::default()
                };
                let style = base_style.add_modifier(if is_hovered {
                    Modifier::REVERSED
                } else {
                    Modifier::empty()
                });
                ListItem::new(Line::from(spans)).style(style)
            })
            .collect();

        List::new(items).render(area, buf);
    }
}

fn viewport(len: usize, hovered: Option<usize>, height: u16) -> (usize, usize) {
    if len == 0 || height == 0 {
        return (0, 0);
    }
    let visible = height as usize;
    let focus = hovered.unwrap_or(0).min(len.saturating_sub(1));
    if len <= visible {
        return (0, len);
    }
    let max_start = len - visible;
    let start = focus.saturating_sub(visible / 2).min(max_start);
    let end = start + visible;
    (start, end)
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_info(names: &[&str], current: Option<&str>) -> BranchInfo {
        BranchInfo {
            branches: names
                .iter()
                .map(|name| BranchSummary {
                    name: (*name).to_string(),
                    ahead: None,
                    behind: None,
                    is_remote: false,
                    remote_ref: None,
                })
                .collect(),
            current: current.map(str::to_string),
            status: None,
            hovered: None,
            selected: None,
        }
    }

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
            is_remote: false,
            remote_ref: None,
        };
        assert_eq!(format_indicator(&branch), "↑2 ↓1");

        branch.ahead = Some(0);
        branch.behind = Some(0);
        assert_eq!(format_indicator(&branch), "↑0 ↓0");

        branch.ahead = None;
        branch.behind = None;
        assert_eq!(format_indicator(&branch), "↑0 ↓0");
    }

    #[test]
    fn indicator_defaults_for_remote_branch() {
        let branch = BranchSummary {
            name: "origin/feature".into(),
            ahead: Some(1),
            behind: Some(1),
            is_remote: true,
            remote_ref: Some("origin/feature".into()),
        };
        assert_eq!(format_indicator(&branch), "↑1 ↓1");
    }

    #[test]
    fn preferred_hover_prefers_previous_selection() {
        let info = make_info(&["main", "feature"], Some("main"));

        assert_eq!(preferred_hover_index(&info, Some(1)), Some(1));
    }

    #[test]
    fn preferred_hover_defaults_to_current_when_no_previous() {
        let info = make_info(&["main", "feature"], Some("main"));

        assert_eq!(preferred_hover_index(&info, None), Some(0));
    }

    #[test]
    fn preferred_hover_clamps_out_of_range_previous() {
        let info = make_info(&["main", "feature"], Some("main"));

        assert_eq!(preferred_hover_index(&info, Some(10)), Some(1));
    }
}
