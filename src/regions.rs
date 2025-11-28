#[derive(Debug, Default, PartialEq)]
pub enum Region {
    #[default]
    Commits,
    Details,
    Branches,
    Stashes,
}

impl Region {
    pub fn as_str(&self) -> &'static str {
        match self {
            Region::Commits => "[c] Commits",
            Region::Branches => "[b] Branches",
            Region::Details => "[d] Details",
            Region::Stashes => "[s] Stashes",
        }
    }
}
