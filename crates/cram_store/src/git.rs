use std::path::{Path, PathBuf};
use std::process::Command;

/// Result of a git sync operation.
#[derive(Debug)]
pub enum SyncResult {
    Pulled(String),
    AlreadyUpToDate,
    NotAGitRepo,
    Error(String),
}

/// Check whether the given path has a `.git` directory directly inside it.
pub fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

/// Walk up from `path` to find the nearest ancestor (or self) containing `.git`.
/// Returns `None` if no git repo is found.
pub fn find_git_root(path: &Path) -> Option<PathBuf> {
    let mut current = path;
    loop {
        if current.join(".git").exists() {
            return Some(current.to_path_buf());
        }
        match current.parent() {
            Some(parent) if parent != current => current = parent,
            _ => return None,
        }
    }
}

/// Run `git pull --ff-only` in the given directory.
/// Walks up to find the git root first, so subdirectories of a repo work.
pub fn pull(path: &Path) -> SyncResult {
    let Some(root) = find_git_root(path) else {
        return SyncResult::NotAGitRepo;
    };

    let result = Command::new("git")
        .args(["pull", "--ff-only"])
        .current_dir(&root)
        .output();

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !output.status.success() {
                return SyncResult::Error(stderr.trim().to_string());
            }

            if stdout.contains("Already up to date") {
                SyncResult::AlreadyUpToDate
            } else {
                SyncResult::Pulled(stdout.trim().to_string())
            }
        }
        Err(e) => SyncResult::Error(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_git_repo_false_for_empty_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(!is_git_repo(dir.path()));
    }

    #[test]
    fn pull_on_non_git_dir_returns_not_a_git_repo() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(matches!(pull(dir.path()), SyncResult::NotAGitRepo));
    }

    #[test]
    fn find_git_root_returns_none_for_non_git_dir() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(find_git_root(dir.path()).is_none());
    }

    #[test]
    fn find_git_root_finds_parent_repo() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join(".git")).expect("mkdir .git");
        let sub = dir.path().join("sub/dir");
        std::fs::create_dir_all(&sub).expect("mkdir sub");
        assert_eq!(find_git_root(&sub), Some(dir.path().to_path_buf()));
    }

    #[test]
    fn find_git_root_finds_self() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::create_dir_all(dir.path().join(".git")).expect("mkdir .git");
        assert_eq!(find_git_root(dir.path()), Some(dir.path().to_path_buf()));
    }
}
