use std::process::Stdio;
use tokio::process::Command;

pub async fn run_aad_tool(args: Vec<String>) -> Result<String, String> {
    let path = find_aad_tool()?;
    run_inner(&path, &[], args, false).await
}

pub async fn run_aad_tool_as_root(args: Vec<String>) -> Result<String, String> {
    let aad_tool = find_aad_tool()?;
    let pkexec = find_pkexec()?;
    run_inner(&pkexec, &[aad_tool.as_str()], args, true).await
}

/// Locate the `aad-tool` binary, preferring absolute paths.
fn find_aad_tool() -> Result<String, String> {
    for path in ["/usr/bin/aad-tool", "/usr/local/bin/aad-tool", "/bin/aad-tool"] {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    Err("aad-tool not found. Please ensure Himmelblau is installed and aad-tool is in /usr/bin.".to_string())
}

/// Locate `pkexec`, preferring the canonical path.
fn find_pkexec() -> Result<String, String> {
    for path in ["/usr/bin/pkexec", "/bin/pkexec"] {
        if std::path::Path::new(path).exists() {
            return Ok(path.to_string());
        }
    }
    Err("pkexec not found. Please install polkit (provides /usr/bin/pkexec).".to_string())
}

async fn run_inner(
    program: &str,
    prefix: &[&str],
    args: Vec<String>,
    is_privileged: bool,
) -> Result<String, String> {
    let output = Command::new(program)
        .args(prefix)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| {
            if is_privileged {
                format!("Failed to launch pkexec — is polkit installed and running? ({e})")
            } else {
                format!("Failed to spawn {program}: {e}")
            }
        })?;

    // pkexec-specific exit codes
    if is_privileged {
        match output.status.code() {
            Some(126) => {
                return Err(
                    "Authentication cancelled or access denied. \
                     No changes were made."
                        .to_string(),
                )
            }
            Some(127) => {
                return Err(
                    "aad-tool not found by pkexec. \
                     Ensure Himmelblau is installed in /usr/bin."
                        .to_string(),
                )
            }
            _ => {}
        }
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let combined = combine(stdout, stderr);

    if output.status.success() {
        Ok(if combined.is_empty() {
            "(completed successfully)".into()
        } else {
            combined
        })
    } else {
        Err(if combined.is_empty() {
            format!(
                "Command failed with exit code {}",
                output.status.code().unwrap_or(-1)
            )
        } else {
            combined
        })
    }
}

fn combine(stdout: String, stderr: String) -> String {
    match (stdout.is_empty(), stderr.is_empty()) {
        (false, false) => format!("{stdout}\n{stderr}"),
        (false, true) => stdout,
        (true, false) => stderr,
        (true, true) => String::new(),
    }
}
