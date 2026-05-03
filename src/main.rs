//! fledge-standup — generate a Markdown standup post from recent git activity.
//!
//! Wraps `fledge ask`, so it inherits whatever AI provider/model the user has
//! configured via `fledge ai use`. Replaces the previous bash script (≤ v0.2.x):
//! same flags and prompt structure, no `python3`/`jq` dependencies.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Stdio};

use anyhow::{bail, Context, Result};
use clap::Parser;
use serde::Deserialize;
use time::{format_description::well_known::Rfc3339, Duration, OffsetDateTime, UtcOffset};

#[derive(Parser, Debug)]
#[command(
    name = "fledge-standup",
    about = "Markdown standup summary from recent git activity",
    long_about = None,
    disable_version_flag = true,
)]
struct Cli {
    /// Time window for commits (default: "24 hours ago"). Anything `git log
    /// --since` accepts: "1 week ago", "yesterday", "2026-04-20", etc.
    #[arg(long, default_value = "24 hours ago")]
    since: String,

    /// Filter to a specific author. Substring against name AND email.
    #[arg(long)]
    author: Option<String>,

    /// Filter to commits by you (email-first, then name).
    #[arg(long)]
    me: bool,

    /// Comma-separated list of repo paths.
    #[arg(long, conflicts_with_all = ["repo_dir", "gh"])]
    repos: Option<String>,

    /// Auto-discover all git repos one level deep under <path>.
    #[arg(long, conflicts_with_all = ["repos", "gh"])]
    repo_dir: Option<PathBuf>,

    /// Use `gh search commits` to fan out across every GitHub-visible repo.
    #[arg(long, conflicts_with_all = ["repos", "repo_dir"])]
    gh: bool,

    /// Restrict --gh search to a specific GitHub username.
    #[arg(long, requires = "gh")]
    gh_user: Option<String>,

    /// Include diff stats in the model's context (single-repo only).
    #[arg(long)]
    include_diff: bool,

    /// Skip the AI step — print the raw aggregated log.
    #[arg(long)]
    raw: bool,

    /// Print the prompt that was sent (debug).
    #[arg(long)]
    show_prompt: bool,

    /// Forwarded to `fledge ask`. Pass after `--`.
    #[arg(last = true)]
    ask_passthrough: Vec<String>,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(err) => {
            eprintln!("fledge standup: {err:#}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode> {
    let mut cli = Cli::parse();

    if cli.me && cli.author.is_none() {
        cli.author = Some(resolve_me()?);
    }

    let scope = build_scope(&cli)?;

    if scope.log.is_empty() {
        return Ok(ExitCode::SUCCESS);
    }

    if cli.raw {
        println!("{}", scope.log);
        return Ok(ExitCode::SUCCESS);
    }

    let prompt = build_prompt(
        &cli,
        &scope.label,
        &scope.log,
        scope.diff_stats.as_deref(),
        &today_str(),
    );

    if cli.show_prompt {
        eprintln!("--- prompt ---");
        eprintln!("{prompt}");
        eprintln!("--- end prompt ---");
        eprintln!();
    }

    let mut command = Command::new("fledge");
    command.arg("ask").arg("--no-spec-index").arg(&prompt);
    for arg in &cli.ask_passthrough {
        command.arg(arg);
    }
    let status = command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to invoke `fledge ask` — is fledge installed and on PATH?")?;
    Ok(match status.code() {
        Some(code) => ExitCode::from((code & 0xff) as u8),
        None => ExitCode::FAILURE,
    })
}

// MARK: - Scope dispatch

struct Scope {
    log: String,
    diff_stats: Option<String>,
    label: String,
}

fn build_scope(cli: &Cli) -> Result<Scope> {
    if cli.gh {
        scope_gh(cli)
    } else if cli.repos.is_some() || cli.repo_dir.is_some() {
        scope_multi(cli)
    } else {
        scope_single(cli)
    }
}

// MARK: - Single repo

fn scope_single(cli: &Cli) -> Result<Scope> {
    if !is_git_repo(Path::new(".")) {
        bail!("not inside a git repository (use --repos, --repo-dir, or --gh)");
    }

    let log = git_log(Path::new("."), &cli.since, cli.author.as_deref())?;
    let label = project_label(Path::new("."));

    if log.is_empty() {
        if let Some(author) = &cli.author {
            eprintln!(
                "fledge standup: no commits by '{author}' since '{since}'.",
                since = cli.since
            );
            let recent = recent_authors(Path::new("."), &cli.since)?;
            if !recent.is_empty() {
                eprintln!("  Recent authors in this window:");
                for name in &recent {
                    eprintln!("    - {name}");
                }
                eprintln!(
                    "  Try: fledge standup --author <name>  (or drop --me/--author for everyone)"
                );
            }
        } else {
            eprintln!(
                "fledge standup: no commits since '{since}'.",
                since = cli.since
            );
        }
        return Ok(Scope {
            log,
            diff_stats: None,
            label,
        });
    }

    let diff_stats = if cli.include_diff {
        Some(git_diff_stats(
            Path::new("."),
            &cli.since,
            cli.author.as_deref(),
        )?)
    } else {
        None
    };

    Ok(Scope {
        log,
        diff_stats,
        label,
    })
}

// MARK: - Multi repo (--repos / --repo-dir)

fn scope_multi(cli: &Cli) -> Result<Scope> {
    let mut paths: Vec<PathBuf> = Vec::new();

    if let Some(repos) = &cli.repos {
        for raw in repos.split(',') {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                paths.push(expand_tilde(trimmed));
            }
        }
    }

    if let Some(repo_dir) = &cli.repo_dir {
        let dir = expand_tilde(&repo_dir.to_string_lossy());
        if !dir.is_dir() {
            bail!("--repo-dir '{}' is not a directory", dir.display());
        }
        let mut found: Vec<PathBuf> = fs::read_dir(&dir)
            .with_context(|| format!("failed to read --repo-dir '{}'", dir.display()))?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| is_git_repo(path))
            .collect();
        found.sort();
        paths.extend(found);
    }

    if paths.is_empty() {
        bail!("no repos to scan");
    }

    let mut combined = String::new();
    let mut repo_count = 0usize;
    for repo in &paths {
        if !is_git_repo(repo) {
            eprintln!(
                "fledge standup: '{}' is not a git repo (skipping).",
                repo.display()
            );
            continue;
        }
        let label = repo_basename(repo);
        let log = git_log(repo, &cli.since, cli.author.as_deref())?;
        if !log.is_empty() {
            combined.push_str(&format!("## {label}\n{log}\n\n"));
            repo_count += 1;
        }
    }

    let log = combined.trim_end().to_string();
    Ok(Scope {
        log,
        diff_stats: None,
        label: format!("{repo_count} repos"),
    })
}

// MARK: - GitHub-wide

#[derive(Deserialize)]
struct GhCommitter {
    date: String,
}

#[derive(Deserialize)]
struct GhCommitInner {
    message: String,
    committer: GhCommitter,
}

#[derive(Deserialize)]
struct GhRepository {
    #[serde(rename = "fullName")]
    full_name: String,
}

#[derive(Deserialize)]
struct GhCommit {
    sha: String,
    commit: GhCommitInner,
    repository: GhRepository,
}

fn scope_gh(cli: &Cli) -> Result<Scope> {
    if which("gh").is_none() {
        bail!("--gh requires the GitHub CLI (gh) to be installed and authenticated");
    }

    let since_iso = since_to_iso_date(&cli.since).with_context(|| {
        format!(
            "--gh can't convert --since '{}' to an ISO date; use a relative form (\"1 week ago\", \"yesterday\") or YYYY-MM-DD",
            cli.since
        )
    })?;

    let gh_author = match (&cli.gh_user, cli.me, &cli.author) {
        (Some(user), _, _) => user.clone(),
        (None, true, _) => "@me".to_string(),
        (None, false, None) => "@me".to_string(),
        (None, false, Some(author)) => author.clone(),
    };

    let output = Command::new("gh")
        .arg("search")
        .arg("commits")
        .arg(format!("--author={gh_author}"))
        .arg(format!("--committer-date=>={since_iso}"))
        .arg("--limit=200")
        .arg("--json")
        .arg("sha,commit,repository")
        .output()
        .context("failed to invoke `gh search commits`")?;
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        bail!("`gh search commits` failed: {}", err.trim());
    }

    let commits: Vec<GhCommit> = serde_json::from_slice(&output.stdout)
        .context("failed to parse `gh search commits` output as JSON")?;
    let label = format!("GitHub-wide (author: {gh_author})");
    if commits.is_empty() {
        eprintln!(
            "fledge standup: no commits found on GitHub for author '{gh_author}' since {since_iso}."
        );
        return Ok(Scope {
            log: String::new(),
            diff_stats: None,
            label,
        });
    }

    let offset = local_offset();
    let mut by_repo: BTreeMap<String, Vec<(String, String, String)>> = BTreeMap::new();
    for entry in commits {
        let short_sha: String = entry.sha.chars().take(7).collect();
        let subject = entry
            .commit
            .message
            .lines()
            .next()
            .unwrap_or("")
            .to_string();
        let date = iso_to_local_date(&entry.commit.committer.date, offset);
        by_repo
            .entry(entry.repository.full_name)
            .or_default()
            .push((date, short_sha, subject));
    }

    let mut log = String::new();
    for (repo, entries) in &by_repo {
        log.push_str(&format!("## {repo}\n"));
        for (date, sha, subject) in entries {
            log.push_str(&format!("{date}  {sha}  {subject}\n"));
        }
        log.push('\n');
    }
    let log = log.trim_end().to_string();

    Ok(Scope {
        log,
        diff_stats: None,
        label,
    })
}

// MARK: - Prompt

fn build_prompt(
    cli: &Cli,
    scope_label: &str,
    log: &str,
    diff_stats: Option<&str>,
    today: &str,
) -> String {
    let author_part = match &cli.author {
        Some(a) => format!(" (author: {a})"),
        None => String::new(),
    };
    let mut prompt = format!(
        "You are turning a list of git commits into a concise, paste-ready Markdown standup post for a team channel.\n\
\n\
Output exactly two sections, in this order, using these headers verbatim:\n\
  ## Yesterday\n\
  ## Today\n\
\n\
Today's date is {today}. Each commit line in the input begins with its date in YYYY-MM-DD form, then the short SHA, then the subject.\n\
\n\
Rules:\n\
- \"Yesterday\" — past-tense bullets for commits whose date is BEFORE {today}. Write 3–8 bullets grouped by theme. Use the commit subjects, rewrite them in past tense, and strip conventional-commit prefixes (feat:, fix:, etc.). When commits span multiple repos (the input may have \"## owner/repo\" headers), call out which repo each bullet belongs to in parentheses, e.g. \"- Bumped tokei to library mode (fledge-plugin-metrics)\".\n\
- \"Today\" — past-tense bullets for commits whose date IS {today}, written the same way as Yesterday (multi-repo annotations included). If there are no commits dated {today}, then instead write 1–3 plausible next-step bullets inferred from the commits, each prefixed with \"(inferred)\" so the reader can edit.\n\
- No preamble. No trailing summary. Just the two sections.\n\
- Total length: 12 lines or fewer.\n\
\n\
Scope: {scope_label}\n\
Window: since {since}{author_part}\n\
\n\
Commits:\n\
{log}",
        since = cli.since,
    );
    if let Some(stats) = diff_stats {
        if !stats.is_empty() {
            prompt.push_str("\n\nDiff stats (per commit):\n");
            prompt.push_str(stats);
        }
    }
    prompt
}

// MARK: - Helpers

fn resolve_me() -> Result<String> {
    if let Some(email) = git_config("user.email")? {
        return Ok(email);
    }
    if let Some(name) = git_config("user.name")? {
        return Ok(name);
    }
    bail!("--me passed but neither user.email nor user.name is set")
}

fn git_config(key: &str) -> Result<Option<String>> {
    let output = Command::new("git").args(["config", key]).output();
    match output {
        Ok(out) if out.status.success() => {
            let value = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Ok((!value.is_empty()).then_some(value))
        }
        _ => Ok(None),
    }
}

fn is_git_repo(path: &Path) -> bool {
    path.join(".git").exists()
}

fn expand_tilde(value: &str) -> PathBuf {
    if let Some(rest) = value.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home).join(rest);
        }
    }
    PathBuf::from(value)
}

fn project_label(path: &Path) -> String {
    let toplevel = Command::new("git")
        .arg("-C")
        .arg(path)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty());
    match toplevel {
        Some(top) => PathBuf::from(top)
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "project".to_string()),
        None => "project".to_string(),
    }
}

fn repo_basename(path: &Path) -> String {
    let canonical = path.canonicalize().ok();
    let source = canonical.as_deref().unwrap_or(path);
    source
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string())
}

fn git_log(path: &Path, since: &str, author: Option<&str>) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(path).arg("log");
    cmd.arg(format!("--since={since}"));
    cmd.arg("--pretty=format:%cd  %h  %s");
    cmd.arg("--date=format-local:%Y-%m-%d");
    if let Some(name) = author {
        cmd.arg(format!("--author={name}"));
    }
    let output = cmd.output().context("failed to run `git log`")?;
    if !output.status.success() {
        return Ok(String::new());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_diff_stats(path: &Path, since: &str, author: Option<&str>) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C").arg(path).arg("log");
    cmd.arg(format!("--since={since}"));
    cmd.arg("--shortstat")
        .arg("--pretty=format:%h")
        .arg("--no-merges");
    if let Some(name) = author {
        cmd.arg(format!("--author={name}"));
    }
    let output = cmd
        .output()
        .context("failed to run `git log --shortstat`")?;
    if !output.status.success() {
        return Ok(String::new());
    }
    let raw = String::from_utf8_lossy(&output.stdout);
    Ok(raw.lines().take(500).collect::<Vec<_>>().join("\n"))
}

fn recent_authors(path: &Path, since: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
        .arg("log")
        .arg(format!("--since={since}"))
        .arg("--pretty=format:%an")
        .output()
        .context("failed to run `git log` for recent authors")?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    let mut names: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();
    names.sort();
    names.dedup();
    names.truncate(5);
    Ok(names)
}

fn which(cmd: &str) -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    std::env::split_paths(&path_var)
        .map(|dir| dir.join(cmd))
        .find(|candidate| candidate.is_file())
}

/// Convert a `git log --since` style spec into a YYYY-MM-DD string suitable
/// for `gh search commits --committer-date=>=…`. Supports `<N> <unit>(s)? ago`,
/// `yesterday`, `today`, and a YYYY-MM-DD passthrough.
fn since_to_iso_date(spec: &str) -> Result<String> {
    let trimmed = spec.trim().to_lowercase();
    let now = OffsetDateTime::now_utc();

    if let Some(stripped) = trimmed.strip_suffix(" ago") {
        let mut parts = stripped.split_whitespace();
        let count_str = parts.next().context("expected number")?;
        let unit = parts.next().context("expected unit")?;
        if parts.next().is_some() {
            bail!("unrecognized --since spec: '{spec}'");
        }
        let count: i64 = count_str.parse().context("expected integer for --since")?;
        let unit_singular = unit.trim_end_matches('s');
        let duration = match unit_singular {
            "hour" => Duration::hours(count),
            "day" => Duration::days(count),
            "week" => Duration::weeks(count),
            "month" => Duration::days(30 * count),
            "year" => Duration::days(365 * count),
            other => bail!("unsupported --since unit: '{other}'"),
        };
        return Ok(format_iso_date(now - duration));
    }

    if trimmed == "yesterday" {
        return Ok(format_iso_date(now - Duration::days(1)));
    }
    if trimmed == "today" {
        return Ok(format_iso_date(now));
    }

    if trimmed.len() >= 10 {
        let head = &trimmed[..10];
        let bytes = head.as_bytes();
        let looks_iso = bytes[..4].iter().all(u8::is_ascii_digit)
            && bytes[4] == b'-'
            && bytes[5..7].iter().all(u8::is_ascii_digit)
            && bytes[7] == b'-'
            && bytes[8..10].iter().all(u8::is_ascii_digit);
        if looks_iso {
            return Ok(head.to_string());
        }
    }

    bail!("can't convert --since '{spec}' to an ISO date")
}

fn format_iso_date(when: OffsetDateTime) -> String {
    let date = when.date();
    format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        date.month() as u8,
        date.day()
    )
}

fn local_offset() -> UtcOffset {
    UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC)
}

fn today_str() -> String {
    let now = OffsetDateTime::now_utc().to_offset(local_offset());
    format_iso_date(now)
}

fn iso_to_local_date(iso: &str, offset: UtcOffset) -> String {
    OffsetDateTime::parse(iso, &Rfc3339)
        .map(|dt| dt.to_offset(offset))
        .map(format_iso_date)
        .unwrap_or_else(|_| iso.chars().take(10).collect())
}
