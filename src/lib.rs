//! Opencase — declarative test-case management.
//!
//! Slices 01–02: case file format + parsing + `validate` rules + `review`
//! state machine. Format contract: see `.scratch/opencase/PRD.md` —
//! "Case file format (contract)".

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const STATUSES: [&str; 2] = ["draft", "reviewed"];
pub const MODES: [&str; 2] = ["manual", "scripted"];
pub const CATEGORIES: [&str; 3] = ["product-bug", "test-bug", "environment"];
pub const REQUIRED: [&str; 5] = ["id", "title", "status", "mode", "source"];

/// A parsed case file. `front` preserves source order for write-back.
pub struct Case {
    pub path: PathBuf,
    pub front: Vec<(String, String)>,
    pub body: String,
    pub records: Vec<Record>,
    /// Record-line parse problems discovered in the body.
    pub errors: Vec<String>,
}

impl Case {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.front
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Replace (or append) a frontmatter field and write the file back.
    pub fn set_field(&mut self, key: &str, value: &str) -> Result<(), String> {
        if let Some(slot) = self.front.iter_mut().find(|(k, _)| k == key) {
            slot.1 = value.to_string();
        } else {
            self.front.push((key.to_string(), value.to_string()));
        }
        self.write()
    }

    /// Replace (or append) the `status` field and write the file back.
    pub fn set_status(&mut self, status: &str) -> Result<(), String> {
        self.set_field("status", status)
    }

    /// Append a record line (without the `- ` prefix) to the `## Executions`
    /// section, creating the section if missing. Records stay contiguous.
    pub fn add_record(&mut self, line: &str) -> Result<(), String> {
        let marker = "## Executions";
        if let Some(pos) = self.body.find(marker) {
            let tail = &self.body[pos + marker.len()..];
            // Insert after the last record line, or after the header's
            // whitespace if the section is empty.
            let mut insert: Option<usize> = None;
            let mut offset = 0usize;
            for l in tail.lines() {
                offset += l.len() + 1;
                if l.starts_with("- ") {
                    insert = Some(offset);
                }
            }
            let abs = match insert {
                Some(i) => pos + marker.len() + i.min(tail.len()),
                None => pos
                    + marker.len()
                    + tail
                        .find(|c: char| !c.is_whitespace())
                        .unwrap_or(tail.len()),
            };
            let mut out = String::with_capacity(self.body.len() + line.len() + 2);
            out.push_str(&self.body[..abs]);
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str("- ");
            out.push_str(line);
            out.push('\n');
            out.push_str(&self.body[abs..]);
            self.body = out;
        } else {
            if !self.body.ends_with('\n') {
                self.body.push('\n');
            }
            self.body.push_str(&format!("\n{marker}\n\n- {line}\n"));
        }
        self.write()
    }

    /// Non-empty lines under `## <name>` in the body.
    pub fn section(&self, name: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut in_sec = false;
        for line in self.body.lines() {
            if line.starts_with("## ") {
                in_sec = line.trim() == format!("## {name}");
                continue;
            }
            if in_sec && !line.trim().is_empty() {
                out.push(line.to_string());
            }
        }
        out
    }

    fn write(&self) -> Result<(), String> {
        let mut out = String::from("---\n");
        for (k, v) in &self.front {
            out.push_str(k);
            out.push_str(": ");
            out.push_str(v);
            out.push('\n');
        }
        out.push_str("---\n");
        out.push_str(&self.body);
        fs::write(&self.path, out).map_err(|e| format!("{}: {e}", self.path.display()))
    }
}

pub struct Record {
    pub date: String,
    pub commit: String,
    pub result: String,
    pub category: Option<String>,
    pub note: Option<String>,
}

pub fn parse_case(path: &Path) -> Result<Case, String> {
    let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let rest = text
        .strip_prefix("---\n")
        .ok_or_else(|| format!("{}: missing frontmatter", path.display()))?;
    let end = rest
        .find("\n---")
        .ok_or_else(|| format!("{}: unterminated frontmatter", path.display()))?;
    let mut front = Vec::new();
    for line in rest[..end].lines() {
        if let Some((k, v)) = line.split_once(':') {
            front.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    let mut case = Case {
        path: path.to_path_buf(),
        front,
        body: rest[end + 4..].strip_prefix('\n').unwrap_or(&rest[end + 4..]).to_string(),
        records: Vec::new(),
        errors: Vec::new(),
    };
    case.parse_records();
    Ok(case)
}

fn is_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b
            .iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

fn parse_record_line(line: &str) -> Result<Record, String> {
    let parts: Vec<&str> = line[2..].split('|').map(str::trim).collect();
    if parts.len() < 3 || parts.len() > 5 {
        return Err(format!("bad record line: {line}"));
    }
    if !is_date(parts[0]) {
        return Err(format!("bad record date '{}': {line}", parts[0]));
    }
    if parts[2] != "pass" && parts[2] != "fail" {
        return Err(format!("bad record result '{}': {line}", parts[2]));
    }
    let category = parts.get(3).map(|s| s.to_string());
    let note = parts.get(4).map(|s| s.to_string());
    if parts[2] == "fail" {
        if !category.as_deref().is_some_and(|c| CATEGORIES.contains(&c)) {
            return Err(format!(
                "failed record needs category {CATEGORIES:?}: {line}"
            ));
        }
    } else if category.is_some() {
        return Err(format!("pass record must not have a category: {line}"));
    }
    Ok(Record {
        date: parts[0].to_string(),
        commit: parts[1].to_string(),
        result: parts[2].to_string(),
        category,
        note,
    })
}

impl Case {
    fn parse_records(&mut self) {
        let mut in_rec = false;
        for line in self.body.lines() {
            if line.starts_with("## ") {
                in_rec = line.trim() == "## Executions";
                continue;
            }
            if !(in_rec && line.starts_with("- ")) {
                continue;
            }
            match parse_record_line(line) {
                Ok(r) => self.records.push(r),
                Err(e) => self.errors.push(format!("{}: {e}", self.path.display())),
            }
        }
    }
}

/// Read and parse every `*.md` in `dir`. Parse failures land in the error
/// list; the caller decides whether to surface them (validate does, review
/// skips broken files).
pub fn load_cases(dir: &Path) -> Result<(Vec<Case>, Vec<String>), String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut cases = Vec::new();
    let mut errs = Vec::new();
    for entry in entries {
        let path = match entry {
            Ok(e) => e.path(),
            Err(e) => {
                errs.push(e.to_string());
                continue;
            }
        };
        if !path.is_file() || path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        match parse_case(&path) {
            Ok(c) => cases.push(c),
            Err(e) => errs.push(e),
        }
    }
    cases.sort_by(|a, b| {
        (a.get("id").unwrap_or(""), &a.path).cmp(&(b.get("id").unwrap_or(""), &b.path))
    });
    Ok((cases, errs))
}

/// Escape `|` for markdown table cells.
fn esc(s: &str) -> String {
    s.replace('|', "\\|")
}

/// `report` command: markdown summary of status, mode, coverage and last run.
pub fn cmd_report(dir: &Path) -> Result<String, String> {
    let (cases, _) = load_cases(dir)?;
    let mut out = format!("# Opencase Report — {}\n", today()?);
    if cases.is_empty() {
        out.push_str("\nno cases\n");
        return Ok(out);
    }
    let total = cases.len();
    let reviewed = cases
        .iter()
        .filter(|c| c.get("status") == Some("reviewed"))
        .count();
    let manual = cases
        .iter()
        .filter(|c| c.get("mode") == Some("manual"))
        .count();
    let covered = cases.iter().filter(|c| c.get("covered-by").is_some()).count();
    out.push_str(&format!(
        "\nTotal: {total} | reviewed: {reviewed} | draft: {}\n\
         Manual: {manual} | scripted: {} | automated coverage: {covered}\n\n",
        total - reviewed,
        total - manual,
    ));
    out.push_str("| case | title | status | mode | covered-by | last run |\n");
    out.push_str("|------|-------|--------|------|------------|----------|\n");
    for c in &cases {
        let last = match c.records.last() {
            Some(r) => format!("{} {}", r.result, r.date),
            None => "—".to_string(),
        };
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            esc(c.get("id").unwrap_or("")),
            esc(c.get("title").unwrap_or("")),
            c.get("status").unwrap_or(""),
            c.get("mode").unwrap_or(""),
            esc(c.get("covered-by").unwrap_or("—")),
            last,
        ));
    }
    let drafts: Vec<&Case> = cases
        .iter()
        .filter(|c| c.get("status") == Some("draft"))
        .collect();
    if !drafts.is_empty() {
        out.push_str("\n## Draft (need review)\n");
        for c in drafts {
            out.push_str(&format!(
                "- {} — {}\n",
                esc(c.get("id").unwrap_or("")),
                esc(c.get("title").unwrap_or(""))
            ));
        }
    }
    Ok(out)
}

/// Validate every case in `dir`. Returns the number of case files found and
/// the list of problems (empty means valid).
pub fn validate_dir(dir: &Path) -> (usize, Vec<String>) {
    let (cases, mut problems) = match load_cases(dir) {
        Ok(x) => x,
        Err(e) => return (0, vec![e]),
    };
    let mut seen: HashMap<String, PathBuf> = HashMap::new();
    for case in &cases {
        let path = &case.path;
        for k in REQUIRED {
            if !case.front.iter().any(|(fk, _)| fk == k) {
                problems.push(format!("{}: missing '{k}'", path.display()));
            }
        }
        if let Some(s) = case.get("status") {
            if !STATUSES.contains(&s) {
                problems.push(format!(
                    "{}: bad status '{s}' (need {STATUSES:?})",
                    path.display()
                ));
            }
        }
        if let Some(m) = case.get("mode") {
            if !MODES.contains(&m) {
                problems.push(format!("{}: bad mode '{m}' (need {MODES:?})", path.display()));
            }
        }
        if let Some(id) = case.get("id") {
            if let Some(prev) = seen.get(id) {
                problems.push(format!(
                    "{}: duplicate id '{id}' (also {})",
                    path.display(),
                    prev.display()
                ));
            } else {
                seen.insert(id.to_string(), path.clone());
            }
        }
        if let Some(cb) = case.get("covered-by") {
            let root = case
                .path
                .parent()
                .and_then(|p| p.parent())
                .unwrap_or_else(|| Path::new("."));
            if !root.join(cb).exists() {
                problems.push(format!(
                    "{}: covered-by file missing: {cb}",
                    path.display()
                ));
            }
        }
        if case.get("mode") == Some("scripted") && case.get("covered-by").is_none() {
            problems.push(format!(
                "{}: scripted case needs 'covered-by'",
                path.display()
            ));
        }
        problems.extend(case.errors.iter().cloned());
    }
    (cases.len(), problems)
}

/// `review` command: list drafts, or `--approve` / `--edit` one case.
/// Approval is the only status-flip path; editing a reviewed case resets it
/// to draft. Returns the message to print.
pub fn cmd_review(
    dir: &Path,
    id: Option<&str>,
    approve: bool,
    edit: bool,
) -> Result<String, String> {
    if approve && edit {
        return Err("use only one of --approve / --edit".to_string());
    }
    let (mut cases, errs) = load_cases(dir)?;
    if id.is_none() && approve {
        return Err("review --approve needs a case id".to_string());
    }
    if id.is_none() && edit {
        return Err("review --edit needs a case id".to_string());
    }
    let Some(id) = id else {
        if cases.is_empty() && !errs.is_empty() {
            return Err(errs.join("\n"));
        }
        let drafts: Vec<&Case> = cases
            .iter()
            .filter(|c| c.get("status") == Some("draft"))
            .collect();
        let mut out = if drafts.is_empty() {
            "no draft cases — everything reviewed".to_string()
        } else {
            drafts
                .iter()
                .map(|c| {
                    format!(
                        "{:<28} {:<32} {}",
                        c.get("id").unwrap_or(""),
                        c.get("title").unwrap_or(""),
                        c.get("source").unwrap_or("")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        let drifted: Vec<&Case> = cases.iter().filter(|c| drift_hint(c).is_some()).collect();
        if !drifted.is_empty() {
            out.push_str("\n\n## Drift (Steps/Expected changed since scriptify)\n");
            for c in drifted {
                out.push_str(&format!(
                    "- {} — {}\n",
                    c.get("id").unwrap_or(""),
                    drift_hint(c).unwrap_or_default()
                ));
            }
        }
        return Ok(out);
    };

    let idx = cases
        .iter()
        .position(|c| c.get("id") == Some(id))
        .ok_or_else(|| format!("no case with id '{id}'"))?;
    let c = &mut cases[idx];
    if approve {
        if c.get("status") == Some("draft") {
            c.set_status("reviewed")?;
            Ok(format!("{id}: approved (reviewed)"))
        } else {
            Ok(format!("{id}: already reviewed"))
        }
    } else if edit {
        let was = c.get("status").unwrap_or("").to_string();
        let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
        let status = std::process::Command::new(&editor)
            .arg(&c.path)
            .status()
            .map_err(|e| format!("editor '{editor}' failed: {e}"))?;
        if !status.success() {
            return Err(format!("editor '{editor}' exited with {status}"));
        }
        if was == "reviewed" {
            c.set_status("draft")?;
            Ok(format!("{id}: edited — status reset to draft"))
        } else {
            Ok(format!("{id}: edited (still draft)"))
        }
    } else {
        let mut msg = format!(
            "{id}: status={} mode={}",
            c.get("status").unwrap_or(""),
            c.get("mode").unwrap_or("")
        );
        if let Some(h) = drift_hint(c) {
            msg.push_str(&format!("\n{h}"));
        }
        Ok(msg)
    }
}

/// Local date as YYYY-MM-DD via `date +%F` (local timezone matters for
/// sign-off records). Fails loudly rather than guessing.
pub fn today() -> Result<String, String> {
    let out = std::process::Command::new("date")
        .arg("+%F")
        .output()
        .map_err(|e| format!("cannot run 'date': {e}"))?;
    if !out.status.success() {
        return Err("cannot determine today's date ('date +%F' failed)".to_string());
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if !is_date(&s) {
        return Err(format!("'date +%F' returned unexpected value '{s}'"));
    }
    Ok(s)
}

/// Short git HEAD hash of the current repo, or "unknown" outside git.
pub fn git_short_sha() -> String {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            String::from_utf8_lossy(&o.stdout).trim().to_string()
        }
        _ => "unknown".to_string(),
    }
}

/// `run` command: print the execution prompt for an agent. Gate: reviewed.
pub fn cmd_run(dir: &Path, id: &str) -> Result<String, String> {
    let (cases, _) = load_cases(dir)?;
    let idx = cases
        .iter()
        .position(|c| c.get("id") == Some(id))
        .ok_or_else(|| format!("no case with id '{id}'"))?;
    let c = &cases[idx];
    if c.get("status") != Some("reviewed") {
        return Err(format!(
            "review gate: '{id}' is not reviewed — run `opencase review {id} --approve` first"
        ));
    }
    let mut out = format!(
        "# Execute: {} (mode: {})\nSource: {}",
        c.get("title").unwrap_or(""),
        c.get("mode").unwrap_or(""),
        c.get("source").unwrap_or("")
    );
    out.push_str("\n\n## Steps\n");
    for s in c.section("Steps") {
        out.push_str(&format!("\n{s}"));
    }
    out.push_str("\n\n## Expected\n");
    for s in c.section("Expected") {
        out.push_str(&format!("\n{s}"));
    }
    out.push_str(&format!(
        "\n\nThen record:\n  opencase record {id} --result pass|fail [--category product-bug|test-bug|environment] [--note ...]"
    ));
    Ok(out)
}

/// `record` command: append a dated execution record. Gates: reviewed +
/// manual; fail requires a category; pass forbids one.
pub fn cmd_record(
    dir: &Path,
    id: &str,
    result: &str,
    category: Option<&str>,
    commit: Option<&str>,
    note: Option<&str>,
) -> Result<String, String> {
    let (mut cases, _) = load_cases(dir)?;
    let idx = cases
        .iter()
        .position(|c| c.get("id") == Some(id))
        .ok_or_else(|| format!("no case with id '{id}'"))?;
    let c = &mut cases[idx];
    if c.get("status") != Some("reviewed") {
        return Err(format!(
            "review gate: '{id}' is not reviewed — cannot record execution"
        ));
    }
    if c.get("mode") != Some("manual") {
        return Err(format!(
            "'{id}' is scripted — scripted results live in CI, opencase only records manual runs"
        ));
    }
    if result != "pass" && result != "fail" {
        return Err(format!("--result must be pass or fail, got '{result}'"));
    }
    if result == "fail" && !category.is_some_and(|c| CATEGORIES.contains(&c)) {
        return Err(format!("failed run needs --category: {CATEGORIES:?}"));
    }
    if result == "pass" && category.is_some() {
        return Err("--category is only for failed runs".to_string());
    }
    if let Some(n) = note {
        if n.contains('|') {
            return Err("--note must not contain '|'".to_string());
        }
    }
    let mut parts = vec![today()?, commit.map(str::to_string).unwrap_or_else(git_short_sha), result.to_string()];
    if let Some(cat) = category {
        parts.push(cat.to_string());
    }
    if let Some(n) = note {
        parts.push(n.to_string());
    }
    c.add_record(&parts.join(" | "))?;
    Ok(format!("{id}: recorded {result}"))
}

/// FNV-1a 64-bit hash of a case's Steps+Expected, hex-encoded. Deterministic
/// and dependency-free — good enough for drift detection (not security).
fn drift_hash(case: &Case) -> String {
    let mut s = String::new();
    for line in case.section("Steps") {
        s.push_str(&line);
        s.push('\n');
    }
    s.push('\0');
    for line in case.section("Expected") {
        s.push_str(&line);
        s.push('\n');
    }
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

/// Drift hint: Some(message) when the case's Steps/Expected changed since
/// the `drift-sha` baseline recorded at scriptify time.
fn drift_hint(case: &Case) -> Option<String> {
    let stored = case.get("drift-sha")?;
    if drift_hash(case) == stored {
        return None;
    }
    Some(format!(
        "⚠ drift: Steps/Expected changed since scriptify — covered-by script may be stale \
         (run `opencase scriptify {} --rebaseline` after updating it)",
        case.get("id").unwrap_or("")
    ))
}

/// Conversion context shared by scriptify and scriptify --rebaseline.
fn scriptify_context(c: &Case) -> String {
    let mut out = format!(
        "# Scriptify: {} (mode: {})\nSource: {}",
        c.get("title").unwrap_or(""),
        c.get("mode").unwrap_or(""),
        c.get("source").unwrap_or("")
    );
    out.push_str("\n\n## Steps\n");
    for s in c.section("Steps") {
        out.push_str(&format!("\n{s}"));
    }
    out.push_str("\n\n## Executions\n");
    for r in &c.records {
        out.push_str(&format!("\n- {} | {} | {}", r.date, r.commit, r.result));
        if let Some(cat) = &r.category {
            out.push_str(&format!(" | {cat}"));
        }
        if let Some(n) = &r.note {
            out.push_str(&format!(" | {n}"));
        }
    }
    out
}

/// `scriptify` command: print the conversion context (steps + execution
/// records) for an agent to turn into a script, then flip the case to
/// scripted with a covered-by link and a drift baseline. Gates: reviewed +
/// manual. `--rebaseline` refreshes the drift baseline of an already-
/// scripted case after its script was updated.
pub fn cmd_scriptify(
    dir: &Path,
    id: &str,
    covered_by: Option<&str>,
    rebaseline: bool,
) -> Result<String, String> {
    let (mut cases, _) = load_cases(dir)?;
    let idx = cases
        .iter()
        .position(|c| c.get("id") == Some(id))
        .ok_or_else(|| format!("no case with id '{id}'"))?;
    let c = &mut cases[idx];
    if c.get("status") != Some("reviewed") {
        return Err(format!("review gate: '{id}' is not reviewed — cannot scriptify"));
    }
    if rebaseline {
        if c.get("mode") != Some("scripted") {
            return Err(format!(
                "'{id}' is not scripted — --rebaseline refreshes the drift baseline of a scripted case"
            ));
        }
        let mut out = scriptify_context(c);
        c.set_field("drift-sha", &drift_hash(c))?;
        out.push_str(
            "\n\nDrift baseline refreshed. Update the script to match the current Steps, then validate.",
        );
        return Ok(out);
    }
    if c.get("mode") == Some("scripted") {
        return Err(format!("'{id}' is already scripted"));
    }
    let cb = covered_by
        .map(str::to_string)
        .unwrap_or_else(|| format!("tests/{id}.spec.ts"));
    let hash = drift_hash(c);
    let mut out = scriptify_context(c);
    out.push_str(&format!(
        "\n\nWrite the script at '{cb}' from the steps above, then re-run validate."
    ));
    c.set_field("mode", "scripted")?;
    c.set_field("covered-by", &cb)?;
    c.set_field("drift-sha", &hash)?;
    Ok(out)
}

/// `init` command: scaffold a cases/ directory with one example case so a
/// non-Rust user can start without cloning this repo.
pub fn cmd_init(dir: &Path) -> Result<String, String> {
    let cases = dir.join("cases");
    if cases.exists() {
        return Err(format!(
            "{} already exists — nothing to do",
            cases.display()
        ));
    }
    fs::create_dir_all(&cases).map_err(|e| format!("{}: {e}", cases.display()))?;
    let example = "---\nid: example-happy-path\ntitle: Example happy path\nstatus: draft\nmode: manual\nsource: <your PRD doc + section>\n---\n\n## Steps\n\n1. <first step>\n2. <second step>\n\n## Expected\n\n- <verifiable expectation, one per bullet>\n";
    fs::write(cases.join("example-happy-path.md"), example)
        .map_err(|e| format!("{}: {e}", cases.join("example-happy-path.md").display()))?;
    Ok(format!(
        "Initialized {} — next: run `opencase review`, then a case-reviewer session before any execution",
        cases.display()
    ))
}

/// Skills embedded in the binary so `opencase skill install` works without
/// the repo — version-locked to the release. Source of truth: skills/.
pub const SKILLS: [(&str, &str); 3] = [
    ("case-writer", include_str!("../skills/case-writer/SKILL.md")),
    ("case-reviewer", include_str!("../skills/case-reviewer/SKILL.md")),
    ("case-executor", include_str!("../skills/case-executor/SKILL.md")),
];

/// `skill install` command: write the embedded skills into the target
/// harness's skills directory. `--dir` overrides the base directory (also
/// used by tests).
pub fn cmd_skill_install(agent: &str, force: bool, dir: Option<&str>) -> Result<String, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME is not set".to_string())?;
    let base = match dir {
        Some(d) => PathBuf::from(d),
        None => PathBuf::from(match agent {
            "pi" => format!("{home}/.agents/skills"),
            "claude" => format!("{home}/.claude/skills"),
            "codex" => format!("{home}/.codex/skills"),
            "project" => ".agents/skills".to_string(),
            other => {
                return Err(format!(
                    "unknown agent '{other}' — valid: pi, claude, codex, project"
                ))
            }
        }),
    };
    let mut installed = Vec::new();
    for (name, content) in SKILLS {
        let d = base.join(name);
        let target = d.join("SKILL.md");
        if target.exists() && !force {
            return Err(format!(
                "{} already exists — re-run with --force to overwrite",
                target.display()
            ));
        }
        fs::create_dir_all(&d).map_err(|e| format!("{}: {e}", d.display()))?;
        fs::write(&target, content).map_err(|e| format!("{}: {e}", target.display()))?;
        installed.push(name.to_string());
    }
    Ok(format!(
        "Installed {} into {} — restart your agent to pick them up",
        installed.join(", "),
        base.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmpdir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("opencase-test-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn write_case(dir: &Path, name: &str, front: &str, body: &str) -> PathBuf {
        let p = dir.join(format!("{name}.md"));
        fs::write(&p, format!("---\n{front}---\n{body}")).unwrap();
        p
    }

    const VALID_FRONT: &str =
        "id: login\ntitle: Login\nstatus: draft\nmode: manual\nsource: PRD\n";

    fn valid_body() -> &'static str {
        "\n## Steps\n\n1. go\n\n## Expected\n\n- ok\n"
    }

    #[test]
    fn parses_valid_case() {
        let d = tmpdir("parse-valid");
        let p = write_case(&d, "login", VALID_FRONT, valid_body());
        let c = parse_case(&p).unwrap();
        assert_eq!(c.get("id"), Some("login"));
        assert_eq!(c.get("status"), Some("draft"));
        assert_eq!(c.get("source"), Some("PRD"));
        assert!(c.records.is_empty());
        assert!(c.errors.is_empty());
    }

    #[test]
    fn rejects_missing_frontmatter() {
        let d = tmpdir("parse-nofront");
        let p = d.join("x.md");
        fs::write(&p, "no frontmatter here\n").unwrap();
        assert!(parse_case(&p).is_err());
    }

    #[test]
    fn rejects_unterminated_frontmatter() {
        let d = tmpdir("parse-unterm");
        let p = d.join("x.md");
        fs::write(&p, "---\nid: x\n").unwrap();
        assert!(parse_case(&p).is_err());
    }

    #[test]
    fn roundtrip_preserves_file() {
        let d = tmpdir("roundtrip");
        let p = write_case(&d, "login", VALID_FRONT, valid_body());
        let before = fs::read_to_string(&p).unwrap();
        let c = parse_case(&p).unwrap();
        c.write().unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), before);
    }

    #[test]
    fn parses_record_lines() {
        let d = tmpdir("records");
        let body = "\n## Executions\n\n- 2025-06-01 | abc123 | pass\n- 2025-06-02 | def456 | fail | product-bug | button unresponsive\n- 2025-06-03 | 111 | fail\n- 2025-06-04 | 222 | pass | test-bug\n- 2025-06-xx | 333 | pass\n";
        let p = write_case(&d, "login", VALID_FRONT, body);
        let c = parse_case(&p).unwrap();
        assert_eq!(c.records.len(), 2);
        assert_eq!(c.records[0].result, "pass");
        assert_eq!(c.records[1].category.as_deref(), Some("product-bug"));
        assert_eq!(c.records[1].note.as_deref(), Some("button unresponsive"));
        assert_eq!(c.errors.len(), 3, "errors: {:?}", c.errors);
    }

    #[test]
    fn record_lines_ignored_outside_executions() {
        let d = tmpdir("records-outside");
        let body = "\n## Steps\n\n- 2025-06-01 | abc | pass\n\n## Expected\n\n- ok\n";
        let p = write_case(&d, "login", VALID_FRONT, body);
        let c = parse_case(&p).unwrap();
        assert!(c.records.is_empty());
        assert!(c.errors.is_empty());
    }

    #[test]
    fn validate_accepts_valid_dir() {
        let d = tmpdir("valid-dir");
        let cases = d.join("cases");
        fs::create_dir_all(&cases).unwrap();
        write_case(&cases, "a", VALID_FRONT, valid_body());
        write_case(
            &cases,
            "b",
            "id: b\ntitle: B\nstatus: reviewed\nmode: scripted\nsource: P\ncovered-by: tests/b.spec.ts\n",
            valid_body(),
        );
        fs::create_dir_all(d.join("tests")).unwrap();
        fs::write(d.join("tests/b.spec.ts"), "// stub\n").unwrap();
        let (count, problems) = validate_dir(&cases);
        assert_eq!(count, 2);
        assert!(problems.is_empty(), "problems: {:?}", problems);
    }

    #[test]
    fn validate_flags_missing_field() {
        let d = tmpdir("missing-field");
        write_case(
            &d,
            "a",
            "id: a\ntitle: A\nmode: manual\nsource: P\n",
            valid_body(),
        );
        let (_, problems) = validate_dir(&d);
        assert!(
            problems.iter().any(|p| p.contains("missing 'status'")),
            "{:?}",
            problems
        );
    }

    #[test]
    fn validate_flags_duplicate_id() {
        let d = tmpdir("dup-id");
        write_case(&d, "a", VALID_FRONT, valid_body());
        write_case(
            &d,
            "b",
            "id: login\ntitle: B\nstatus: draft\nmode: manual\nsource: P\n",
            valid_body(),
        );
        let (_, problems) = validate_dir(&d);
        assert!(
            problems.iter().any(|p| p.contains("duplicate id 'login'")),
            "{:?}",
            problems
        );
    }

    #[test]
    fn validate_flags_bad_status_and_mode() {
        let d = tmpdir("bad-enums");
        write_case(
            &d,
            "a",
            "id: a\ntitle: A\nstatus: shipped\nmode: manual\nsource: P\n",
            valid_body(),
        );
        write_case(
            &d,
            "b",
            "id: b\ntitle: B\nstatus: draft\nmode: magical\nsource: P\n",
            valid_body(),
        );
        let (_, problems) = validate_dir(&d);
        assert!(
            problems.iter().any(|p| p.contains("bad status 'shipped'")),
            "{:?}",
            problems
        );
        assert!(
            problems.iter().any(|p| p.contains("bad mode 'magical'")),
            "{:?}",
            problems
        );
    }

    #[test]
    fn validate_flags_bad_records() {
        let d = tmpdir("bad-records");
        let body = "\n## Executions\n\n- 2025-06-01 | abc | fail\n";
        write_case(&d, "a", VALID_FRONT, body);
        let (_, problems) = validate_dir(&d);
        assert!(
            problems.iter().any(|p| p.contains("failed record needs category")),
            "{:?}",
            problems
        );
    }

    #[test]
    fn validate_missing_dir() {
        let (count, problems) = validate_dir(Path::new("/nonexistent/opencase-test-dir"));
        assert_eq!(count, 0);
        assert_eq!(problems.len(), 1);
    }

    // --- review state machine ---

    fn reviewed_front(name: &str) -> String {
        format!("id: {name}\ntitle: {name}\nstatus: reviewed\nmode: manual\nsource: P\n")
    }

    fn front_for(name: &str) -> String {
        format!("id: {name}\ntitle: {name}\nstatus: draft\nmode: manual\nsource: P\n")
    }

    #[test]
    fn review_lists_drafts_only() {
        let d = tmpdir("review-list");
        write_case(&d, "a", &reviewed_front("a"), valid_body());
        write_case(&d, "b", &front_for("b"), valid_body());
        let out = cmd_review(&d, None, false, false).unwrap();
        assert!(out.contains("b"), "out: {out}");
        assert!(!out.contains("a"), "out: {out}");
    }

    #[test]
    fn review_no_drafts_message() {
        let d = tmpdir("review-nodrafts");
        write_case(&d, "a", &reviewed_front("a"), valid_body());
        let out = cmd_review(&d, None, false, false).unwrap();
        assert!(out.contains("no draft cases"), "out: {out}");
    }

    #[test]
    fn review_approve_flips_to_reviewed() {
        let d = tmpdir("review-approve");
        let p = write_case(&d, "b", &front_for("b"), valid_body());
        let out = cmd_review(&d, Some("b"), true, false).unwrap();
        assert!(out.contains("approved"), "out: {out}");
        let text = fs::read_to_string(&p).unwrap();
        assert!(text.contains("status: reviewed"), "text: {text}");
    }

    #[test]
    fn review_approve_reviewed_is_noop() {
        let d = tmpdir("review-approve-noop");
        let p = write_case(&d, "a", &reviewed_front("a"), valid_body());
        let out = cmd_review(&d, Some("a"), true, false).unwrap();
        assert!(out.contains("already reviewed"), "out: {out}");
        assert!(fs::read_to_string(&p).unwrap().contains("status: reviewed"));
    }

    #[test]
    fn review_edit_resets_reviewed_to_draft() {
        let d = tmpdir("review-edit-reset");
        let p = write_case(&d, "a", &reviewed_front("a"), valid_body());
        std::env::set_var("EDITOR", "true");
        let out = cmd_review(&d, Some("a"), false, true).unwrap();
        assert!(out.contains("reset to draft"), "out: {out}");
        let text = fs::read_to_string(&p).unwrap();
        assert!(text.contains("status: draft"), "text: {text}");
    }

    #[test]
    fn review_edit_draft_stays_draft() {
        let d = tmpdir("review-edit-draft");
        let p = write_case(&d, "b", &front_for("b"), valid_body());
        std::env::set_var("EDITOR", "true");
        let out = cmd_review(&d, Some("b"), false, true).unwrap();
        assert!(out.contains("still draft"), "out: {out}");
        assert!(fs::read_to_string(&p).unwrap().contains("status: draft"));
    }

    #[test]
    fn review_unknown_id_and_flag_mix() {
        let d = tmpdir("review-errors");
        write_case(&d, "a", VALID_FRONT, valid_body());
        assert!(cmd_review(&d, Some("nope"), true, false).is_err());
        assert!(cmd_review(&d, Some("a"), true, true).is_err());
        assert!(cmd_review(&d, None, true, false).is_err());
    }

    // --- run + record ---

    fn reviewed_manual(name: &str) -> String {
        format!("id: {name}\ntitle: {name}\nstatus: reviewed\nmode: manual\nsource: P\n")
    }

    #[test]
    fn run_refused_on_draft() {
        let d = tmpdir("run-draft");
        write_case(&d, "a", &front_for("a"), valid_body());
        let err = cmd_run(&d, "a").unwrap_err();
        assert!(err.contains("review gate"), "err: {err}");
    }

    #[test]
    fn run_prints_prompt() {
        let d = tmpdir("run-prompt");
        write_case(&d, "a", &reviewed_manual("a"), valid_body());
        let out = cmd_run(&d, "a").unwrap();
        assert!(out.contains("Source: P"));
        assert!(out.contains("1. go"));
        assert!(out.contains("- ok"));
    }

    #[test]
    fn record_refused_on_draft() {
        let d = tmpdir("rec-draft");
        write_case(&d, "a", &front_for("a"), valid_body());
        let err = cmd_record(&d, "a", "pass", None, Some("c1"), None).unwrap_err();
        assert!(err.contains("review gate"), "err: {err}");
    }

    #[test]
    fn record_refused_on_scripted() {
        let d = tmpdir("rec-scripted");
        write_case(
            &d,
            "a",
            "id: a\ntitle: A\nstatus: reviewed\nmode: scripted\nsource: P\ncovered-by: tests/a.spec.ts\n",
            valid_body(),
        );
        let err = cmd_record(&d, "a", "pass", None, Some("c1"), None).unwrap_err();
        assert!(err.contains("scripted"), "err: {err}");
    }

    #[test]
    fn record_pass_appends_line() {
        let d = tmpdir("rec-pass");
        let p = write_case(&d, "a", &reviewed_manual("a"), valid_body());
        cmd_record(&d, "a", "pass", None, Some("abc123"), None).unwrap();
        let text = fs::read_to_string(&p).unwrap();
        assert!(text.contains("## Executions"), "text: {text}");
        assert!(text.contains("- 20"), "text: {text}"); // date prefix
        let c = parse_case(&p).unwrap();
        assert_eq!(c.records.len(), 1);
        assert_eq!(c.records[0].result, "pass");
        assert_eq!(c.records[0].commit, "abc123");
        assert!(c.errors.is_empty(), "{:?}", c.errors);
    }

    #[test]
    fn record_fail_requires_category() {
        let d = tmpdir("rec-fail-nocat");
        write_case(&d, "a", &reviewed_manual("a"), valid_body());
        let err = cmd_record(&d, "a", "fail", None, Some("c1"), None).unwrap_err();
        assert!(err.contains("--category"), "err: {err}");
        let err = cmd_record(&d, "a", "fail", Some("bogus"), Some("c1"), None).unwrap_err();
        assert!(err.contains("--category"), "err: {err}");
    }

    #[test]
    fn record_fail_with_category_appends() {
        let d = tmpdir("rec-fail-cat");
        let p = write_case(&d, "a", &reviewed_manual("a"), valid_body());
        cmd_record(&d, "a", "fail", Some("product-bug"), Some("c1"), Some("button broken")).unwrap();
        let text = fs::read_to_string(&p).unwrap();
        assert!(text.contains("fail | product-bug | button broken"), "text: {text}");
    }

    #[test]
    fn record_pass_rejects_category() {
        let d = tmpdir("rec-pass-cat");
        write_case(&d, "a", &reviewed_manual("a"), valid_body());
        let err = cmd_record(&d, "a", "pass", Some("test-bug"), Some("c1"), None).unwrap_err();
        assert!(err.contains("only for failed"), "err: {err}");
    }

    #[test]
    fn record_note_rejects_pipe() {
        let d = tmpdir("rec-pipe");
        write_case(&d, "a", &reviewed_manual("a"), valid_body());
        let err = cmd_record(&d, "a", "pass", None, Some("c1"), Some("a|b")).unwrap_err();
        assert!(err.contains("'|'"), "err: {err}");
    }

    #[test]
    fn record_appends_in_order() {
        let d = tmpdir("rec-order");
        let p = write_case(&d, "a", &reviewed_manual("a"), valid_body());
        cmd_record(&d, "a", "pass", None, Some("c1"), None).unwrap();
        cmd_record(&d, "a", "fail", Some("test-bug"), Some("c2"), Some("wrong expectation")).unwrap();
        let c = parse_case(&p).unwrap();
        assert_eq!(c.records.len(), 2);
        assert_eq!(c.records[0].commit, "c1");
        assert_eq!(c.records[1].result, "fail");
        assert_eq!(c.records[1].category.as_deref(), Some("test-bug"));
    }

    #[test]
    fn record_appends_into_existing_section() {
        let d = tmpdir("rec-existing");
        let p = write_case(
            &d,
            "a",
            &reviewed_manual("a"),
            "\n## Steps\n\n1. go\n\n## Expected\n\n- ok\n\n## Executions\n\n- 2025-06-01 | old | pass\n",
        );
        cmd_record(&d, "a", "pass", None, Some("new"), None).unwrap();
        let c = parse_case(&p).unwrap();
        assert_eq!(c.records.len(), 2);
        assert_eq!(c.records[0].commit, "old");
        assert_eq!(c.records[1].commit, "new");
    }

    #[test]
    fn validate_flags_missing_covered_by_file() {
        let d = tmpdir("cb-missing");
        write_case(
            &d,
            "a",
            "id: a\ntitle: A\nstatus: reviewed\nmode: manual\nsource: P\ncovered-by: tests/a.spec.ts\n",
            valid_body(),
        );
        let (_, problems) = validate_dir(&d);
        assert!(
            problems.iter().any(|p| p.contains("covered-by file missing")),
            "{:?}",
            problems
        );
    }

    #[test]
    fn validate_flags_scripted_without_covered_by() {
        let d = tmpdir("cb-scripted");
        write_case(
            &d,
            "a",
            "id: a\ntitle: A\nstatus: reviewed\nmode: scripted\nsource: P\n",
            valid_body(),
        );
        let (_, problems) = validate_dir(&d);
        assert!(
            problems.iter().any(|p| p.contains("scripted case needs 'covered-by'")),
            "{:?}",
            problems
        );
    }

    #[test]
    fn scriptify_refused_on_draft() {
        let d = tmpdir("sc-draft");
        write_case(&d, "a", &front_for("a"), valid_body());
        let err = cmd_scriptify(&d, "a", None, false).unwrap_err();
        assert!(err.contains("review gate"), "err: {err}");
    }

    #[test]
    fn scriptify_refused_when_already_scripted() {
        let d = tmpdir("sc-scripted");
        write_case(
            &d,
            "a",
            "id: a\ntitle: A\nstatus: reviewed\nmode: scripted\nsource: P\ncovered-by: tests/a.spec.ts\n",
            valid_body(),
        );
        let err = cmd_scriptify(&d, "a", None, false).unwrap_err();
        assert!(err.contains("already scripted"), "err: {err}");
    }

    #[test]
    fn scriptify_flips_case_and_passes_validate() {
        let d = tmpdir("sc-flip");
        let cases = d.join("cases");
        fs::create_dir_all(&cases).unwrap();
        let p = write_case(&cases, "a", &reviewed_manual("a"), valid_body());
        cmd_record(&cases, "a", "pass", None, Some("c1"), None).unwrap();
        cmd_record(&cases, "a", "fail", Some("test-bug"), Some("c2"), Some("wrong expectation")).unwrap();
        fs::create_dir_all(d.join("tests")).unwrap();
        fs::write(d.join("tests/a.spec.ts"), "// stub\n").unwrap();
        let out = cmd_scriptify(&cases, "a", None, false).unwrap();
        assert!(out.contains("1. go"), "out: {out}");
        assert!(out.contains("fail | test-bug"), "out: {out}");
        let text = fs::read_to_string(&p).unwrap();
        assert!(text.contains("mode: scripted"), "text: {text}");
        assert!(text.contains("covered-by: tests/a.spec.ts"), "text: {text}");
        let (_, problems) = validate_dir(&cases);
        assert!(problems.is_empty(), "{:?}", problems);
    }

    #[test]
    fn scriptify_custom_covered_by() {
        let d = tmpdir("sc-custom");
        let cases = d.join("cases");
        fs::create_dir_all(&cases).unwrap();
        write_case(&cases, "a", &reviewed_manual("a"), valid_body());
        fs::create_dir_all(d.join("e2e")).unwrap();
        fs::write(d.join("e2e/a.spec.ts"), "// stub\n").unwrap();
        let out = cmd_scriptify(&cases, "a", Some("e2e/a.spec.ts"), false).unwrap();
        assert!(out.contains("e2e/a.spec.ts"), "out: {out}");
        let (_, problems) = validate_dir(&cases);
        assert!(problems.is_empty(), "{:?}", problems);
    }

    #[test]
    fn run_allows_scripted_case() {
        let d = tmpdir("run-scripted");
        write_case(
            &d,
            "a",
            "id: a\ntitle: A\nstatus: reviewed\nmode: scripted\nsource: P\ncovered-by: tests/a.spec.ts\n",
            valid_body(),
        );
        let out = cmd_run(&d, "a").unwrap();
        assert!(out.contains("mode: scripted"), "out: {out}");
    }

    // --- drift hint ---

    fn scripted_dir(name: &str) -> (PathBuf, PathBuf) {
        let d = tmpdir(name);
        let cases = d.join("cases");
        fs::create_dir_all(&cases).unwrap();
        fs::create_dir_all(d.join("tests")).unwrap();
        fs::write(d.join("tests/a.spec.ts"), "// stub\n").unwrap();
        (d, cases)
    }

    #[test]
    fn scriptify_sets_drift_baseline() {
        let (d, cases) = scripted_dir("drift-baseline");
        write_case(&cases, "a", &reviewed_manual("a"), valid_body());
        cmd_scriptify(&cases, "a", None, false).unwrap();
        let c = parse_case(&cases.join("a.md")).unwrap();
        assert!(c.get("drift-sha").is_some(), "front: {:?}", c.front);
        assert!(drift_hint(&c).is_none());
    }

    #[test]
    fn drift_hint_fires_when_steps_change() {
        let (d, cases) = scripted_dir("drift-change");
        let p = write_case(&cases, "a", &reviewed_manual("a"), valid_body());
        cmd_scriptify(&cases, "a", None, false).unwrap();
        // change Steps after scriptify (simulating a requirement edit)
        let text = fs::read_to_string(&p).unwrap().replace("1. go", "1. NEW STEP");
        fs::write(&p, text).unwrap();
        let c = parse_case(&p).unwrap();
        assert!(drift_hint(&c).is_some());
        let out = cmd_review(&cases, Some("a"), false, false).unwrap();
        assert!(out.contains("drift"), "out: {out}");
        let out = cmd_review(&cases, None, false, false).unwrap();
        assert!(out.contains("## Drift"), "out: {out}");
        assert!(out.contains("--rebaseline"), "out: {out}");
    }

    #[test]
    fn no_drift_hint_when_content_unchanged() {
        let (d, cases) = scripted_dir("drift-clean");
        let p = write_case(&cases, "a", &reviewed_manual("a"), valid_body());
        cmd_scriptify(&cases, "a", None, false).unwrap();
        // appending an execution record touches the file but not Steps/Expected
        // (record refuses scripted cases by design, so append like add_record does)
        let mut text = fs::read_to_string(&p).unwrap();
        text.push_str("\n## Executions\n\n- 2026-08-08 | c1 | pass\n");
        fs::write(&p, text).unwrap();
        let c = parse_case(&p).unwrap();
        assert!(drift_hint(&c).is_none());
    }

    #[test]
    fn rebaseline_clears_drift() {
        let (d, cases) = scripted_dir("drift-rebase");
        let p = write_case(&cases, "a", &reviewed_manual("a"), valid_body());
        cmd_scriptify(&cases, "a", None, false).unwrap();
        let text = fs::read_to_string(&p).unwrap().replace("1. go", "1. NEW STEP");
        fs::write(&p, text).unwrap();
        let c = parse_case(&p).unwrap();
        assert!(drift_hint(&c).is_some());
        cmd_scriptify(&cases, "a", None, true).unwrap();
        let c = parse_case(&p).unwrap();
        assert!(drift_hint(&c).is_none());
        // rebaseline refuses non-scripted cases
        let d2 = tmpdir("drift-rebase-manual");
        let cases2 = d2.join("cases");
        fs::create_dir_all(&cases2).unwrap();
        write_case(&cases2, "b", &reviewed_manual("b"), valid_body());
        assert!(cmd_scriptify(&cases2, "b", None, true).is_err());
    }

    // --- init ---

    #[test]
    fn init_scaffolds_valid_cases_dir() {
        let d = tmpdir("init-fresh");
        let out = cmd_init(&d).unwrap();
        assert!(out.contains("Initialized"), "out: {out}");
        assert!(d.join("cases/example-happy-path.md").exists());
        let (count, problems) = validate_dir(&d.join("cases"));
        assert_eq!(count, 1);
        assert!(problems.is_empty(), "{:?}", problems);
    }

    #[test]
    fn init_refuses_existing_cases_dir() {
        let d = tmpdir("init-exists");
        fs::create_dir_all(d.join("cases")).unwrap();
        let err = cmd_init(&d).unwrap_err();
        assert!(err.contains("already exists"), "err: {err}");
    }

    // --- skill install ---

    #[test]
    fn skill_install_writes_embedded_skills() {
        let d = tmpdir("skill-install");
        let out = cmd_skill_install("project", false, Some(d.to_str().unwrap())).unwrap();
        assert!(out.contains("case-writer"), "out: {out}");
        for (name, content) in SKILLS {
            let p = d.join(name).join("SKILL.md");
            assert!(p.exists(), "missing {}", p.display());
            assert_eq!(fs::read_to_string(&p).unwrap(), content);
        }
    }

    #[test]
    fn skill_install_refuses_overwrite_without_force() {
        let d = tmpdir("skill-force");
        cmd_skill_install("project", false, Some(d.to_str().unwrap())).unwrap();
        let err = cmd_skill_install("project", false, Some(d.to_str().unwrap())).unwrap_err();
        assert!(err.contains("already exists"), "err: {err}");
        cmd_skill_install("project", true, Some(d.to_str().unwrap())).unwrap();
    }

    #[test]
    fn skill_install_rejects_unknown_agent() {
        let err = cmd_skill_install("vim", false, None).unwrap_err();
        assert!(err.contains("unknown agent"), "err: {err}");
    }

    // --- report ---

    #[test]
    fn report_empty_dir() {
        let d = tmpdir("report-empty");
        let out = cmd_report(&d).unwrap();
        assert!(out.contains("no cases"), "out: {out}");
    }

    #[test]
    fn report_counts_table_and_drafts() {
        let d = tmpdir("report-mixed");
        write_case(&d, "b", &reviewed_manual("b"), valid_body());
        write_case(&d, "a", &front_for("a"), valid_body());
        write_case(
            &d,
            "c",
            "id: c\ntitle: C\nstatus: reviewed\nmode: scripted\nsource: P\ncovered-by: tests/c.spec.ts\n",
            valid_body(),
        );
        let p = write_case(&d, "d", &reviewed_manual("d"), valid_body());
        cmd_record(&d, "d", "fail", Some("product-bug"), Some("abc"), Some("broken")).unwrap();
        let out = cmd_report(&d).unwrap();
        assert!(
            out.contains("Total: 4 | reviewed: 3 | draft: 1"),
            "out: {out}"
        );
        assert!(
            out.contains("Manual: 3 | scripted: 1 | automated coverage: 1"),
            "out: {out}"
        );
        assert!(out.contains("|------|"), "out: {out}");
        assert!(out.contains("fail 20"), "out: {out}");
        assert!(out.contains("| b | b | reviewed | manual | — | — |"), "out: {out}");
        assert!(out.contains("## Draft (need review)"), "out: {out}");
        assert!(out.contains("- a — a"), "out: {out}");
        assert!(!out.contains("- b —"), "out: {out}");
    }
}
