use std::process::Command;

fn main() {
    let git_info = get_git_info();

    match git_info {
        Some((sha, branch)) => {
            println!("cargo:rustc-env=GIT_SHA={}", sha);
            println!("cargo:rustc-env=GIT_BRANCH={}", branch);
            println!("cargo:rustc-env=GIT_AVAILABLE=true");
        }
        None => {
            println!("cargo:rustc-env=GIT_SHA=unknown");
            println!("cargo:rustc-env=GIT_BRANCH=unknown");
            println!("cargo:rustc-env=GIT_AVAILABLE=false");
        }
    }

    // Rebuild if git HEAD or branch changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
}

fn get_git_info() -> Option<(String, String)> {
    // Check if we're in a git repository
    if !is_git_repo() {
        return None;
    }

    let sha = get_git_sha()?;
    let branch = get_git_branch()?;

    Some((sha, branch))
}

fn is_git_repo() -> bool {
    Command::new("git")
        .args(&["rev-parse", "--git-dir"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn get_git_sha() -> Option<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    } else {
        None
    }
}

fn get_git_branch() -> Option<String> {
    // Try to get the current branch name
    let output = Command::new("git")
        .args(&["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        let branch = String::from_utf8(output.stdout)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())?;

        // Handle detached HEAD state
        if branch == "HEAD" {
            // Try to get a tag or describe the commit
            get_git_describe().or(Some("detached".to_string()))
        } else {
            Some(branch)
        }
    } else {
        None
    }
}

fn get_git_describe() -> Option<String> {
    let output = Command::new("git")
        .args(&["describe", "--tags", "--exact-match"])
        .output()
        .ok()?;

    if output.status.success() {
        String::from_utf8(output.stdout)
            .ok()
            .map(|s| format!("tag:{}", s.trim()))
    } else {
        None
    }
}
