use std::{collections::HashSet, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub id: String,
    pub summary: String,
    pub branches: Vec<String>,
}

#[derive(Debug, Default)]
pub struct BranchInfo {
    pub branches: Vec<String>,
    pub current: Option<String>,
    pub status: Option<String>,
    pub hovered: Option<usize>,
    pub selected: Option<String>,
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

fn try_fetch_branch_info(path: impl AsRef<Path>) -> Result<BranchInfo, String> {
    let repo = gix::discover(path).map_err(|err| format!("Not a git repository: {err}"))?;

    let current = repo
        .head()
        .map_err(|err| format!("Failed to read HEAD: {err}"))?
        .referent_name()
        .map(|name| name.shorten().to_string());

    let mut branches: Vec<String> = repo
        .references()
        .map_err(|err| format!("Failed to list references: {err}"))?
        .prefixed("refs/heads/")
        .map_err(|err| format!("Failed to filter branches: {err}"))?
        .filter_map(|reference| reference.ok().map(|r| r.name().shorten().to_string()))
        .collect();

    branches.sort();

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
        path::PathBuf,
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
            info.branches,
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
            info.branches,
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

        assert_eq!(info.branches, vec!["main".to_string()]);
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
