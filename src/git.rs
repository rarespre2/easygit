use std::{collections::HashSet, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub id: String,
    pub summary: String,
    pub branches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BranchSummary {
    pub name: String,
    pub ahead: Option<usize>,
    pub behind: Option<usize>,
}

#[derive(Debug, Default)]
pub struct BranchInfo {
    pub branches: Vec<BranchSummary>,
    pub current: Option<String>,
    pub status: Option<String>,
    pub hovered: Option<usize>,
    pub selected: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    TypeChange,
    Untracked,
    Unmerged,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: String,
    pub change: ChangeType,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RepoStatus {
    pub changes: Vec<FileChange>,
    pub error: Option<String>,
}

impl RepoStatus {
    pub fn total_changes(&self) -> usize {
        self.changes.len()
    }

    pub fn is_clean(&self) -> bool {
        self.error.is_none() && self.changes.is_empty()
    }
}

pub fn fetch_branch_info() -> BranchInfo {
    fetch_branch_info_in(".")
}

pub fn fetch_branch_info_in(path: impl AsRef<Path>) -> BranchInfo {
    match try_fetch_branch_info(path) {
        Ok(info) => info,
        Err(err) => BranchInfo {
            branches: Vec::new(),
            current: None,
            status: Some(err),
            hovered: None,
            selected: None,
        },
    }
}

pub fn checkout_branch(branch: &str) -> Result<(), String> {
    checkout_branch_in(".", branch)
}

pub fn checkout_branch_in(path: impl AsRef<Path>, branch: &str) -> Result<(), String> {
    let status = std::process::Command::new("git")
        .arg("checkout")
        .arg(branch)
        .current_dir(path.as_ref())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|err| format!("Failed to run git checkout: {err}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("git checkout exited with status: {status}"))
    }
}

pub fn create_branch(branch: &str) -> Result<(), String> {
    create_branch_in(".", branch)
}

pub fn create_branch_in(path: impl AsRef<Path>, branch: &str) -> Result<(), String> {
    if branch.trim().is_empty() {
        return Err("Branch name cannot be empty".to_string());
    }

    let output = std::process::Command::new("git")
        .arg("checkout")
        .arg("-b")
        .arg(branch)
        .current_dir(path.as_ref())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|err| format!("Failed to run git checkout -b: {err}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let message = String::from_utf8_lossy(&output.stderr);
        let trimmed = message.trim();
        if trimmed.is_empty() {
            Err(format!(
                "git checkout -b exited with status: {}",
                output.status
            ))
        } else {
            Err(format!("Failed to create branch {branch}: {trimmed}"))
        }
    }
}

pub fn delete_branch(branch: &str) -> Result<(), String> {
    delete_branch_in(".", branch)
}

pub fn delete_branch_in(path: impl AsRef<Path>, branch: &str) -> Result<(), String> {
    let output = std::process::Command::new("git")
        .arg("branch")
        .arg("-d")
        .arg(branch)
        .current_dir(path.as_ref())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|err| format!("Failed to run git branch -d: {err}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let message = String::from_utf8_lossy(&output.stderr);
        let trimmed = message.trim();
        if trimmed.is_empty() {
            Err(format!(
                "git branch -d exited with status: {}",
                output.status
            ))
        } else {
            Err(format!("Failed to delete branch {branch}: {trimmed}"))
        }
    }
}

pub fn pull_current_branch() -> Result<(), String> {
    pull_current_branch_in(".")
}

pub fn pull_current_branch_in(path: impl AsRef<Path>) -> Result<(), String> {
    let output = std::process::Command::new("git")
        .arg("pull")
        .arg("--ff-only")
        .current_dir(path.as_ref())
        .output()
        .map_err(|err| format!("Failed to run git pull: {err}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let message = String::from_utf8_lossy(&output.stderr);
        let trimmed = message.trim();
        if trimmed.is_empty() {
            Err(format!("git pull exited with status: {}", output.status))
        } else {
            Err(trimmed.to_string())
        }
    }
}

pub fn push_current_branch() -> Result<(), String> {
    push_current_branch_in(".")
}

pub fn push_current_branch_in(path: impl AsRef<Path>) -> Result<(), String> {
    let path = path.as_ref();
    let branch = current_branch_name_in(path)
        .ok_or_else(|| "Failed to read current branch name".to_string())?;
    let has_upstream = upstream_for_branch(path, &branch).is_some();

    let mut cmd = std::process::Command::new("git");
    cmd.current_dir(path);
    if has_upstream {
        cmd.arg("push");
    } else {
        cmd.arg("push")
            .arg("--set-upstream")
            .arg("origin")
            .arg(&branch);
    }

    let output = cmd
        .output()
        .map_err(|err| format!("Failed to run git push: {err}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let message = String::from_utf8_lossy(&output.stderr);
        let trimmed = message.trim();
        if trimmed.is_empty() {
            Err(format!("git push exited with status: {}", output.status))
        } else {
            Err(trimmed.to_string())
        }
    }
}

pub fn fetch_commits() -> Result<Vec<Commit>, String> {
    fetch_commits_in(".")
}

pub fn fetch_commits_in(path: impl AsRef<Path>) -> Result<Vec<Commit>, String> {
    let main_branch = find_main_branch_in(path.as_ref());
    let main_commits = main_branch
        .as_deref()
        .map(|name| commits_in_branch(path.as_ref(), name))
        .transpose()?
        .unwrap_or_default();

    let output = std::process::Command::new("git")
        .arg("log")
        .arg("--all")
        .arg("--pretty=format:%H%x09%h%x09%s")
        .current_dir(path.as_ref())
        .output()
        .map_err(|err| format!("Failed to run git log: {err}"))?;

    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr);
        let trimmed = message.trim();
        return if trimmed.is_empty() {
            Err(format!("git log exited with status: {}", output.status))
        } else {
            Err(trimmed.to_string())
        };
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut commits = Vec::new();
    for line in stdout.lines() {
        let mut parts = line.splitn(3, '\t');
        let full_id = parts.next().unwrap_or("").trim();
        let short_id = parts.next().unwrap_or("").trim();
        let summary = parts.next().unwrap_or("").trim().to_string();
        if full_id.is_empty() || short_id.is_empty() {
            continue;
        }

        let mut branches = Vec::new();
        if let Some(main) = &main_branch {
            if main_commits.contains(full_id) {
                branches.push(main.clone());
            }
        }

        if branches.is_empty() {
            let mut containing = branches_containing_commit(path.as_ref(), full_id)?;
            if let Some(main) = &main_branch {
                containing.retain(|b| b != main);
            }
            branches = containing;
        }

        commits.push(Commit {
            id: short_id.to_string(),
            summary,
            branches,
        });
    }

    Ok(commits)
}

pub fn fetch_repo_status() -> RepoStatus {
    fetch_repo_status_in(".")
}

pub fn fetch_repo_status_in(path: impl AsRef<Path>) -> RepoStatus {
    match try_fetch_repo_status(path) {
        Ok(changes) => RepoStatus {
            changes,
            error: None,
        },
        Err(err) => RepoStatus {
            changes: Vec::new(),
            error: Some(err),
        },
    }
}

fn try_fetch_repo_status(path: impl AsRef<Path>) -> Result<Vec<FileChange>, String> {
    let output = std::process::Command::new("git")
        .arg("status")
        .arg("--porcelain=v1")
        .arg("--untracked-files=all")
        .current_dir(path.as_ref())
        .output()
        .map_err(|err| format!("Failed to run git status: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let trimmed = stderr.trim();
        return Err(if trimmed.is_empty() {
            format!("git status exited with status: {}", output.status)
        } else {
            trimmed.to_string()
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let changes = stdout
        .lines()
        .filter_map(parse_status_line)
        .collect::<Vec<_>>();
    Ok(changes)
}

fn parse_status_line(line: &str) -> Option<FileChange> {
    if line.len() < 3 {
        return None;
    }
    let code = &line[0..2];
    let mut path = line[3..].trim();
    if path.is_empty() {
        return None;
    }
    if let Some((_, new)) = path.split_once(" -> ") {
        path = new.trim();
    }
    Some(FileChange {
        path: path.to_string(),
        change: change_type_from_code(code),
    })
}

fn change_type_from_code(code: &str) -> ChangeType {
    if code == "??" {
        return ChangeType::Untracked;
    }

    let mut chars = code.chars();
    let x = chars.next().unwrap_or(' ');
    let y = chars.next().unwrap_or(' ');

    let flag = if x != ' ' { x } else { y };
    match flag {
        'A' => ChangeType::Added,
        'M' => ChangeType::Modified,
        'D' => ChangeType::Deleted,
        'R' => ChangeType::Renamed,
        'C' => ChangeType::Copied,
        'T' => ChangeType::TypeChange,
        'U' => ChangeType::Unmerged,
        '?' => ChangeType::Untracked,
        _ => ChangeType::Unknown,
    }
}

fn find_main_branch_in(path: impl AsRef<Path>) -> Option<String> {
    if branch_exists_in(path.as_ref(), "main") {
        Some("main".to_string())
    } else if branch_exists_in(path.as_ref(), "master") {
        Some("master".to_string())
    } else {
        None
    }
}

fn commits_in_branch(path: &Path, branch: &str) -> Result<HashSet<String>, String> {
    let output = std::process::Command::new("git")
        .arg("rev-list")
        .arg(branch)
        .current_dir(path)
        .output()
        .map_err(|err| format!("Failed to list commits for {branch}: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "git rev-list exited with status {}: {}",
            output.status,
            stderr.trim()
        ));
    }

    let set = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();
    Ok(set)
}

fn branch_exists_in(path: &Path, branch: &str) -> bool {
    std::process::Command::new("git")
        .arg("show-ref")
        .arg("--verify")
        .arg(format!("refs/heads/{branch}"))
        .current_dir(path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn branches_containing_commit(path: &Path, full_id: &str) -> Result<Vec<String>, String> {
    let output = std::process::Command::new("git")
        .arg("branch")
        .arg("--contains")
        .arg(full_id)
        .arg("--format=%(refname:short)")
        .current_dir(path)
        .output()
        .map_err(|err| format!("Failed to find containing branches: {err}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "git branch --contains exited with status {}: {}",
            output.status,
            stderr.trim()
        ));
    }

    let mut branches = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| {
            let trimmed = line.trim_start_matches("* ").trim();
            trimmed.to_string()
        })
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    branches.sort();
    branches.dedup();
    Ok(branches)
}

fn branch_ahead_behind(
    path: &Path,
    branch: &str,
    default_branch: Option<&str>,
) -> Option<(usize, usize)> {
    let target = upstream_for_branch(path, branch).or_else(|| {
        default_branch
            .filter(|candidate| *candidate != branch)
            .map(|name| name.to_string())
    })?;

    ahead_behind_for_pair(path, branch, &target)
}

fn ahead_behind_for_pair(path: &Path, branch: &str, target: &str) -> Option<(usize, usize)> {
    let output = std::process::Command::new("git")
        .arg("rev-list")
        .arg("--left-right")
        .arg("--count")
        .arg(format!("{branch}...{target}"))
        .current_dir(path)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut parts = stdout.trim().split_whitespace();
    let ahead = parts.next()?.parse().ok()?;
    let behind = parts.next().unwrap_or("0").parse().ok()?;
    Some((ahead, behind))
}

fn upstream_for_branch(path: &Path, branch: &str) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg(format!("{branch}@{{upstream}}"))
        .current_dir(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn current_branch_name_in(path: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--abbrev-ref")
        .arg("HEAD")
        .current_dir(path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn try_fetch_branch_info(path: impl AsRef<Path>) -> Result<BranchInfo, String> {
    let path = path.as_ref();
    let repo = gix::discover(path).map_err(|err| format!("Not a git repository: {err}"))?;

    let current = repo
        .head()
        .map_err(|err| format!("Failed to read HEAD: {err}"))?
        .referent_name()
        .map(|name| name.shorten().to_string());

    let mut branches: Vec<BranchSummary> = repo
        .references()
        .map_err(|err| format!("Failed to list references: {err}"))?
        .prefixed("refs/heads/")
        .map_err(|err| format!("Failed to filter branches: {err}"))?
        .filter_map(|reference| {
            reference.ok().map(|r| BranchSummary {
                name: r.name().shorten().to_string(),
                ahead: None,
                behind: None,
            })
        })
        .collect();

    branches.sort();
    let default_branch = find_main_branch_in(path);
    for branch in branches.iter_mut() {
        if let Some((ahead, behind)) =
            branch_ahead_behind(path, &branch.name, default_branch.as_deref())
        {
            branch.ahead = Some(ahead);
            branch.behind = Some(behind);
        }
    }

    Ok(BranchInfo {
        branches,
        current,
        status: None,
        hovered: None,
        selected: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs::{self, File},
        io::Write,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn fetch_lists_all_branches_and_current() {
        let repo = TestRepo::init().unwrap();
        repo.write_file("README.md", "hello").unwrap();
        repo.git(&["add", "."]).unwrap();
        repo.git(&["commit", "-m", "init"]).unwrap();
        repo.git(&["branch", "feature"]).unwrap();

        let info = fetch_branch_info_in(repo.path());

        assert_eq!(info.current.as_deref(), Some("main"));
        assert_eq!(
            branch_names(&info),
            vec!["feature".to_string(), "main".to_string()]
        );
    }

    #[test]
    fn checkout_moves_head_to_requested_branch() {
        let repo = TestRepo::init().unwrap();
        repo.write_file("file.txt", "content").unwrap();
        repo.git(&["add", "."]).unwrap();
        repo.git(&["commit", "-m", "init"]).unwrap();
        repo.git(&["branch", "topic"]).unwrap();

        checkout_branch_in(repo.path(), "topic").unwrap();
        let info = fetch_branch_info_in(repo.path());

        assert_eq!(info.current.as_deref(), Some("topic"));
    }

    #[test]
    fn create_branch_creates_and_checks_out_new_branch() {
        let repo = TestRepo::init().unwrap();
        repo.write_file("file.txt", "content").unwrap();
        repo.git(&["add", "."]).unwrap();
        repo.git(&["commit", "-m", "init"]).unwrap();

        create_branch_in(repo.path(), "feature").unwrap();
        let info = fetch_branch_info_in(repo.path());

        assert_eq!(info.current.as_deref(), Some("feature"));
        assert_eq!(
            branch_names(&info),
            vec!["feature".to_string(), "main".to_string()]
        );
    }

    #[test]
    fn delete_branch_removes_branch_when_not_current() {
        let repo = TestRepo::init().unwrap();
        repo.write_file("file.txt", "content").unwrap();
        repo.git(&["add", "."]).unwrap();
        repo.git(&["commit", "-m", "init"]).unwrap();
        repo.git(&["branch", "old"]).unwrap();

        delete_branch_in(repo.path(), "old").unwrap();
        let info = fetch_branch_info_in(repo.path());

        assert_eq!(branch_names(&info), vec!["main".to_string()]);
    }

    #[test]
    fn delete_branch_fails_when_branch_is_current() {
        let repo = TestRepo::init().unwrap();
        repo.write_file("file.txt", "content").unwrap();
        repo.git(&["add", "."]).unwrap();
        repo.git(&["commit", "-m", "init"]).unwrap();

        let err = delete_branch_in(repo.path(), "main").unwrap_err();
        assert!(err.contains("delete"), "unexpected error: {err}");
    }

    #[test]
    fn fetch_commits_lists_recent_commits() {
        let repo = TestRepo::init().unwrap();
        repo.write_file("file.txt", "first").unwrap();
        repo.git(&["add", "."]).unwrap();
        repo.git(&["commit", "-m", "first"]).unwrap();

        repo.write_file("file.txt", "second").unwrap();
        repo.git(&["add", "."]).unwrap();
        repo.git(&["commit", "-m", "second"]).unwrap();

        let commits = fetch_commits_in(repo.path()).unwrap();

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].summary, "second");
        assert_eq!(commits[1].summary, "first");
        assert!(!commits[0].id.is_empty());
        assert_eq!(commits[0].branches, vec!["main".to_string()]);
    }

    #[test]
    fn fetch_commits_marks_branch_tips() {
        let repo = TestRepo::init().unwrap();
        repo.write_file("file.txt", "base").unwrap();
        repo.git(&["add", "."]).unwrap();
        repo.git(&["commit", "-m", "base"]).unwrap();

        repo.git(&["checkout", "-b", "feature"]).unwrap();
        repo.write_file("file.txt", "feature work").unwrap();
        repo.git(&["add", "."]).unwrap();
        repo.git(&["commit", "-m", "feature work"]).unwrap();

        let commits = fetch_commits_in(repo.path()).unwrap();

        assert_eq!(commits[0].summary, "feature work");
        assert_eq!(commits[0].branches, vec!["feature".to_string()]);
        assert!(
            commits
                .iter()
                .any(|c| c.summary == "base" && c.branches == vec!["main".to_string()])
        );
    }

    fn branch_names(info: &BranchInfo) -> Vec<String> {
        info.branches.iter().map(|b| b.name.clone()).collect()
    }

    #[test]
    fn fetches_ahead_behind_counts_when_upstream_set() {
        let repo = TestRepo::init().unwrap();
        repo.write_file("file.txt", "content").unwrap();
        repo.git(&["add", "."]).unwrap();
        repo.git(&["commit", "-m", "init"]).unwrap();
        repo.git(&["branch", "feature"]).unwrap();

        repo.git(&["checkout", "feature"]).unwrap();
        repo.write_file("file.txt", "feature change").unwrap();
        repo.git(&["commit", "-am", "feature"]).unwrap();

        repo.git(&["checkout", "main"]).unwrap();
        repo.write_file("file.txt", "main change").unwrap();
        repo.git(&["commit", "-am", "main"]).unwrap();

        repo.git(&["branch", "--set-upstream-to=main", "feature"])
            .unwrap();

        let info = fetch_branch_info_in(repo.path());
        let feature = info
            .branches
            .iter()
            .find(|b| b.name == "feature")
            .expect("feature branch");
        assert_eq!(feature.ahead, Some(1));
        assert_eq!(feature.behind, Some(1));
    }

    #[test]
    fn falls_back_to_main_when_upstream_missing() {
        let repo = TestRepo::init().unwrap();
        repo.write_file("file.txt", "content").unwrap();
        repo.git(&["add", "."]).unwrap();
        repo.git(&["commit", "-m", "init"]).unwrap();
        repo.git(&["branch", "feature"]).unwrap();

        repo.git(&["checkout", "feature"]).unwrap();
        repo.write_file("file.txt", "feature change").unwrap();
        repo.git(&["commit", "-am", "feature work"]).unwrap();

        repo.git(&["checkout", "main"]).unwrap();
        repo.write_file("file.txt", "main change").unwrap();
        repo.git(&["commit", "-am", "main work"]).unwrap();

        let info = fetch_branch_info_in(repo.path());
        let feature = info
            .branches
            .iter()
            .find(|b| b.name == "feature")
            .expect("feature branch");
        assert_eq!(feature.ahead, Some(1));
        assert_eq!(feature.behind, Some(1));
    }

    #[test]
    fn pushes_current_branch_to_remote() {
        let repo = TestRepo::init().unwrap();
        repo.write_file("file.txt", "content").unwrap();
        repo.git(&["add", "."]).unwrap();
        repo.git(&["commit", "-m", "init"]).unwrap();

        let remote = create_bare_repo().unwrap();
        repo.add_remote("origin", &remote).unwrap();

        push_current_branch_in(repo.path()).unwrap();

        let status = Command::new("git")
            .arg("--git-dir")
            .arg(remote.to_str().unwrap())
            .arg("show-ref")
            .arg("--verify")
            .arg("refs/heads/main")
            .status()
            .expect("git show-ref");
        assert!(status.success(), "expected remote to have main branch");
        let _ = std::fs::remove_dir_all(remote);
    }

    #[test]
    fn pulls_latest_changes_from_remote() {
        let repo = TestRepo::init().unwrap();
        repo.write_file("file.txt", "content").unwrap();
        repo.git(&["add", "."]).unwrap();
        repo.git(&["commit", "-m", "init"]).unwrap();

        let remote = create_bare_repo().unwrap();
        repo.add_remote("origin", &remote).unwrap();
        push_current_branch_in(repo.path()).unwrap();

        Command::new("git")
            .arg("--git-dir")
            .arg(remote.to_str().unwrap())
            .arg("symbolic-ref")
            .arg("HEAD")
            .arg("refs/heads/main")
            .status()
            .expect("set remote HEAD");

        let clone_path = clone_repo(&remote).unwrap();
        let clone_file = clone_path.join("file.txt");
        let mut contents = std::fs::read_to_string(&clone_file).expect("read clone file");
        contents.push_str("\nremote change");
        std::fs::write(&clone_file, contents).expect("write clone file");

        Command::new("git")
            .arg("-C")
            .arg(clone_path.as_os_str())
            .arg("add")
            .arg(".")
            .status()
            .expect("git add");
        Command::new("git")
            .arg("-C")
            .arg(clone_path.as_os_str())
            .arg("commit")
            .arg("-m")
            .arg("remote change")
            .status()
            .expect("git commit");
        Command::new("git")
            .arg("-C")
            .arg(clone_path.as_os_str())
            .arg("push")
            .status()
            .expect("git push");

        pull_current_branch_in(repo.path()).unwrap();
        let contents = std::fs::read_to_string(repo.path().join("file.txt")).unwrap();
        assert!(
            contents.contains("remote change"),
            "pull should bring remote change"
        );

        let _ = std::fs::remove_dir_all(remote);
        let _ = std::fs::remove_dir_all(clone_path);
    }

    #[test]
    fn parses_porcelain_lines_into_changes() {
        let modified = parse_status_line(" M src/main.rs").unwrap();
        assert_eq!(
            modified,
            FileChange {
                path: "src/main.rs".to_string(),
                change: ChangeType::Modified
            }
        );

        let renamed = parse_status_line("R  old.rs -> new.rs").unwrap();
        assert_eq!(
            renamed,
            FileChange {
                path: "new.rs".to_string(),
                change: ChangeType::Renamed
            }
        );

        let untracked = parse_status_line("?? notes.txt").unwrap();
        assert_eq!(
            untracked,
            FileChange {
                path: "notes.txt".to_string(),
                change: ChangeType::Untracked
            }
        );
    }

    #[test]
    fn fetch_repo_status_lists_local_changes() {
        let repo = TestRepo::init().unwrap();
        repo.write_file("file.txt", "content").unwrap();
        repo.git(&["add", "."]).unwrap();
        repo.git(&["commit", "-m", "init"]).unwrap();

        repo.write_file("file.txt", "updated").unwrap();
        repo.write_file("new.txt", "new file").unwrap();

        let status = fetch_repo_status_in(repo.path());

        assert_eq!(status.total_changes(), 2);
        assert!(
            status
                .changes
                .iter()
                .any(|change| change.path == "file.txt" && change.change == ChangeType::Modified)
        );
        assert!(
            status
                .changes
                .iter()
                .any(|change| change.path == "new.txt" && change.change == ChangeType::Untracked)
        );
        assert!(status.error.is_none());
    }

    fn create_bare_repo() -> Result<PathBuf, String> {
        let path = unique_path("remote");
        let status = Command::new("git")
            .arg("init")
            .arg("--bare")
            .arg(&path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status()
            .map_err(|err| format!("Failed to run git init --bare: {err}"))?;
        if status.success() {
            Ok(path)
        } else {
            Err("git init --bare failed".into())
        }
    }

    fn clone_repo(remote: &Path) -> Result<PathBuf, String> {
        let path = unique_path("clone");
        let status = Command::new("git")
            .arg("clone")
            .arg(remote.as_os_str())
            .arg(&path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status()
            .map_err(|err| format!("Failed to run git clone: {err}"))?;
        if status.success() {
            Ok(path)
        } else {
            Err("git clone failed".into())
        }
    }

    fn unique_path(label: &str) -> PathBuf {
        let mut root = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        root.push(format!(
            "easygit-test-{label}-{nanos}-{}",
            std::process::id()
        ));
        root
    }

    struct TestRepo {
        root: PathBuf,
    }

    impl TestRepo {
        fn init() -> Result<Self, String> {
            let mut root = std::env::temp_dir();
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|err| err.to_string())?
                .as_nanos();
            root.push(format!("easygit-test-{nanos}-{}", std::process::id()));
            fs::create_dir_all(&root).map_err(|err| format!("Failed to create dir: {err}"))?;

            let status = Command::new("git")
                .arg("init")
                .arg("-b")
                .arg("main")
                .current_dir(&root)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped())
                .status()
                .map_err(|err| format!("Failed to run git init: {err}"))?;

            if !status.success() {
                return Err("git init failed".into());
            }

            Ok(Self { root })
        }

        fn git(&self, args: &[&str]) -> Result<(), String> {
            let output = Command::new("git")
                .args(args)
                .current_dir(&self.root)
                .output()
                .map_err(|err| format!("Failed to run git {:?}: {err}", args))?;

            if output.status.success() {
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("git {:?} failed: {}", args, stderr.trim()))
            }
        }

        fn write_file(&self, name: &str, contents: &str) -> Result<(), String> {
            let mut path = self.root.clone();
            path.push(name);
            let mut file = File::create(&path)
                .map_err(|err| format!("Failed to create file {name}: {err}"))?;
            file.write_all(contents.as_bytes())
                .map_err(|err| format!("Failed to write file {name}: {err}"))
        }

        fn add_remote(&self, name: &str, path: &Path) -> Result<(), String> {
            let remote = path
                .to_str()
                .ok_or_else(|| "Remote path is not valid UTF-8".to_string())?;
            self.git(&["remote", "add", name, remote])
        }

        fn path(&self) -> &Path {
            &self.root
        }
    }

    impl Drop for TestRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}
