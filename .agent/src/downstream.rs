use std::fs;
use std::path::Path;
use std::process::{Command, Output};

use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;

use crate::workspace::Workspace;

const POLICY_VERSION: u64 = 1;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Policy {
    schema_version: u64,
    upstream: Upstream,
    allowed_paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Upstream {
    remote: String,
    url: String,
    branch: String,
}

pub fn validate(
    workspace: &Workspace,
    policy_path: &Path,
    upstream_ref: Option<&str>,
) -> Result<()> {
    let policy_path = workspace.existing_inside(policy_path)?;
    let policy_text = fs::read_to_string(&policy_path)
        .with_context(|| format!("cannot read downstream policy {}", policy_path.display()))?;
    let policy: Policy = serde_json::from_str(&policy_text)
        .with_context(|| format!("invalid downstream policy {}", policy_path.display()))?;
    validate_policy(&policy)?;

    let remote_url_key = format!("remote.{}.url", policy.upstream.remote);
    let actual_url = git_text(
        workspace,
        &["config", "--local", "--get-all", &remote_url_key],
    )?;
    ensure!(
        actual_url == policy.upstream.url,
        "remote {:?} points to {actual_url:?}; expected {:?}",
        policy.upstream.remote,
        policy.upstream.url
    );

    let default_ref = format!(
        "refs/remotes/{}/{}",
        policy.upstream.remote, policy.upstream.branch
    );
    let upstream_ref = upstream_ref.unwrap_or(&default_ref);
    validate_ref(upstream_ref)?;
    git_success(
        workspace,
        &[
            "rev-parse",
            "--verify",
            &format!("{upstream_ref}^{{commit}}"),
        ],
    )?;

    let ancestor = git(
        workspace,
        &["merge-base", "--is-ancestor", upstream_ref, "HEAD"],
    )?;
    ensure!(
        ancestor.status.success(),
        "HEAD does not contain {upstream_ref}; merge and validate upstream before publishing the downstream"
    );

    let output = git(
        workspace,
        &[
            "diff",
            "--name-only",
            "--diff-filter=ACDMRTUXB",
            "-z",
            upstream_ref,
            "HEAD",
            "--",
        ],
    )?;
    ensure_git_success(&output, "git diff")?;
    let changed = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| {
            String::from_utf8(path.to_vec()).context("Git returned a non-UTF-8 downstream path")
        })
        .collect::<Result<Vec<_>>>()?;
    let forbidden = changed
        .iter()
        .filter(|path| !policy.allowed_paths.iter().any(|rule| allows(rule, path)))
        .cloned()
        .collect::<Vec<_>>();
    ensure!(
        forbidden.is_empty(),
        "downstream modifies upstream-owned paths: {}",
        forbidden.join(", ")
    );

    println!(
        "Downstream boundary passed: {} owned path(s) differ from {upstream_ref}.",
        changed.len()
    );
    Ok(())
}

fn validate_policy(policy: &Policy) -> Result<()> {
    ensure!(
        policy.schema_version == POLICY_VERSION,
        "unsupported downstream policy schema_version {}",
        policy.schema_version
    );
    validate_atom("upstream.remote", &policy.upstream.remote)?;
    validate_atom("upstream.branch", &policy.upstream.branch)?;
    ensure!(
        policy.upstream.url.starts_with("https://github.com/")
            && Path::new(&policy.upstream.url)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("git")),
        "upstream.url must be an explicit HTTPS GitHub URL ending in .git"
    );
    ensure!(
        !policy.allowed_paths.is_empty(),
        "allowed_paths must not be empty"
    );
    for rule in &policy.allowed_paths {
        validate_rule(rule)?;
    }
    Ok(())
}

fn validate_atom(label: &str, value: &str) -> Result<()> {
    ensure!(!value.is_empty(), "{label} must not be empty");
    ensure!(
        !value.starts_with('-')
            && value
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "-._/".contains(character)),
        "{label} contains unsupported characters"
    );
    Ok(())
}

fn validate_ref(value: &str) -> Result<()> {
    validate_atom("upstream ref", value)
}

fn validate_rule(rule: &str) -> Result<()> {
    ensure!(
        !rule.is_empty()
            && !rule.starts_with('/')
            && !rule.starts_with('-')
            && !rule.contains('\\')
            && !rule.split('/').any(|component| component == ".."),
        "invalid allowed path rule {rule:?}"
    );
    Ok(())
}

fn allows(rule: &str, path: &str) -> bool {
    rule.strip_suffix('/').map_or(path == rule, |prefix| {
        path.starts_with(&format!("{prefix}/"))
    })
}

fn git(workspace: &Workspace, args: &[&str]) -> Result<Output> {
    Command::new("git")
        .args(args)
        .current_dir(workspace.root())
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))
}

fn git_success(workspace: &Workspace, args: &[&str]) -> Result<()> {
    let output = git(workspace, args)?;
    ensure_git_success(&output, &format!("git {}", args.join(" ")))
}

fn git_text(workspace: &Workspace, args: &[&str]) -> Result<String> {
    let output = git(workspace, args)?;
    ensure_git_success(&output, &format!("git {}", args.join(" ")))?;
    String::from_utf8(output.stdout)
        .context("Git returned non-UTF-8 text")
        .map(|text| text.trim().to_owned())
}

fn ensure_git_success(output: &Output, operation: &str) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }
    let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    bail!("{operation} failed: {detail}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    const TEST_UPSTREAM: &str = "https://github.com/corbet-libs/ccvl.git";

    fn git_at(root: &Path, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap().trim().to_owned()
    }

    #[test]
    fn directory_rules_match_descendants_only() {
        assert!(allows("interview/", "interview/profile.md"));
        assert!(!allows("interview/", "interview-private/profile.md"));
        assert!(!allows("interview/", "interview"));
    }

    #[test]
    fn file_rules_are_exact() {
        assert!(allows(
            ".github/workflows/upstream-sync.yml",
            ".github/workflows/upstream-sync.yml"
        ));
        assert!(!allows(
            ".github/workflows/upstream-sync.yml",
            ".github/workflows/other.yml"
        ));
    }

    #[test]
    fn traversal_rules_are_rejected() {
        assert!(validate_rule("../private/").is_err());
        assert!(validate_rule("interview/").is_ok());
    }

    #[test]
    fn repository_boundary_rejects_an_upstream_owned_change() {
        let directory = tempdir().unwrap();
        let root = directory.path();
        fs::write(root.join("ccvl.json"), "{}\n").unwrap();
        fs::write(root.join("README.md"), "upstream\n").unwrap();
        git_at(root, &["init"]);
        git_at(root, &["config", "user.name", "ccvl test"]);
        git_at(root, &["config", "user.email", "ccvl@example.invalid"]);
        git_at(root, &["add", "ccvl.json", "README.md"]);
        git_at(root, &["commit", "-m", "upstream"]);
        let upstream_commit = git_at(root, &["rev-parse", "HEAD"]);
        git_at(
            root,
            &["update-ref", "refs/remotes/upstream/main", &upstream_commit],
        );
        git_at(root, &["remote", "add", "upstream", TEST_UPSTREAM]);
        git_at(
            root,
            &[
                "config",
                "url.git@github.com:.insteadOf",
                "https://github.com/",
            ],
        );
        assert_eq!(
            git_at(root, &["remote", "get-url", "upstream"]),
            "git@github.com:corbet-libs/ccvl.git"
        );

        fs::create_dir(root.join("interview")).unwrap();
        fs::write(root.join("interview/profile.md"), "private profile\n").unwrap();
        fs::write(
            root.join("ccvl-downstream.json"),
            concat!(
                "{\n",
                "  \"schema_version\": 1,\n",
                "  \"upstream\": {\n",
                "    \"remote\": \"upstream\",\n",
                "    \"url\": \"https://github.com/corbet-libs/ccvl.git\",\n",
                "    \"branch\": \"main\"\n",
                "  },\n",
                "  \"allowed_paths\": [\"ccvl-downstream.json\", \"interview/\"]\n",
                "}\n"
            ),
        )
        .unwrap();
        git_at(
            root,
            &["add", "ccvl-downstream.json", "interview/profile.md"],
        );
        git_at(root, &["commit", "-m", "private delta"]);

        let workspace = Workspace::at(root).unwrap();
        validate(
            &workspace,
            Path::new("ccvl-downstream.json"),
            Some("refs/remotes/upstream/main"),
        )
        .unwrap();

        fs::write(root.join("README.md"), "forbidden downstream edit\n").unwrap();
        git_at(root, &["add", "README.md"]);
        git_at(root, &["commit", "-m", "forbidden"]);
        let error = validate(
            &workspace,
            Path::new("ccvl-downstream.json"),
            Some("refs/remotes/upstream/main"),
        )
        .unwrap_err();
        assert!(error.to_string().contains("README.md"));
    }
}
