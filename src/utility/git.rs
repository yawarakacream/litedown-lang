use std::process::Command;

use anyhow::{Context, Result};

pub fn get_current_git_version() -> Result<String> {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .context("failed to run git command")?;
    let hash = String::from_utf8(output.stdout).context("failed to parse output")?;
    Ok(hash)
}
