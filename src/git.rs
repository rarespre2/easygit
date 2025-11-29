use std::path::Path;

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
