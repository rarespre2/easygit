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

    pub fn color(&self, is_selected: bool) -> Color {
        if is_selected {
            Color::Green
        } else {
            Color::Yellow
        }
    }
}
