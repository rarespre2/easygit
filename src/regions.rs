use ratatui::style::Color;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Region {
    #[default]
    Commits,
    Details,
    Branches,
    Stashes,
    Changes,
    ChangeViewer,
    CommitMessage,
}

impl Region {
    pub fn as_str(&self) -> &'static str {
        match self {
            Region::Commits => "[c] Commits",
            Region::Branches => "[b] Branches",
            Region::Details => "[d] Details",
            Region::Stashes => "[s] Stashes",
            Region::Changes => "[c] Changes",
            Region::ChangeViewer => "[v] Change viewer",
            Region::CommitMessage => "[m] Commit message",
        }
    }

    pub fn instructions(&self) -> Vec<&'static str> {
        match self {
            Region::Branches => vec![
                "[↑↓] move",
                "[Enter] checkout",
                "[u] update",
                "[p] push",
                "[a] add",
                "[x] delete",
            ],
            Region::Commits => vec!["[↑↓] move"],
            Region::Changes => vec!["[↑↓] move", "[Enter] stage/unstage", "[x] discard"],
            Region::CommitMessage => vec!["[Enter] commit", "[Esc] stop"],
            Region::Details | Region::Stashes | Region::ChangeViewer => Vec::new(),
        }
    }

    pub fn color(&self, is_selected: bool) -> Color {
        if is_selected {
            Color::Green
        } else {
            Color::Yellow
        }
    }
}
