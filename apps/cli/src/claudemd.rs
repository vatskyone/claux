use anyhow::{bail, Result};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const JUNK_DIRS: &[&str] = &[
    ".git",
    ".idea",
    ".vscode",
    "node_modules",
    "dist",
    "build",
    "target",
    ".next",
    ".turbo",
    "vendor",
    "Pods",
    "DerivedData",
    "__pycache__",
    ".pytest_cache",
    ".venv",
    "venv",
];

#[derive(Debug, Clone, Serialize)]
pub struct GeneratedClaudeMd {
    pub project_path: String,
    pub language: String,
    pub framework: Option<String>,
    pub key_dirs: Vec<String>,
    pub content: String,
}

pub fn generate_for_project(project_path: &str) -> Result<GeneratedClaudeMd> {
    let root = PathBuf::from(project_path);
    if !root.exists() {
        bail!("project path does not exist: {}", project_path);
    }
    if !root.is_dir() {
        bail!("project path is not a directory: {}", project_path);
    }

    let info = inspect_project(&root)?;
    let content = render_markdown(&root, &info);

    Ok(GeneratedClaudeMd {
        project_path: root.display().to_string(),
        language: info.language,
        framework: info.framework,
        key_dirs: info.key_dirs,
        content,
    })
}

pub fn write_claudemd(project_path: &str, content: &str, force: bool) -> Result<PathBuf> {
    let path = claudemd_path(project_path);
    if path.exists() && !force {
        bail!(
            "{} already exists. Use --force to overwrite.",
            path.display()
        );
    }
    fs::write(&path, content)?;
    Ok(path)
}

pub fn claudemd_path(project_path: &str) -> PathBuf {
    PathBuf::from(project_path).join("CLAUDE.md")
}

pub fn read_claudemd(project_path: &str) -> Result<String> {
    let path = claudemd_path(project_path);
    if !path.exists() {
        bail!(
            "{} does not exist. Run `claux claudemd generate --write` first.",
            path.display()
        );
    }
    Ok(fs::read_to_string(path)?)
}

pub fn improve_for_project(project_path: &str) -> Result<GeneratedClaudeMd> {
    let root = PathBuf::from(project_path);
    if !root.exists() {
        bail!("project path does not exist: {}", project_path);
    }
    if !root.is_dir() {
        bail!("project path is not a directory: {}", project_path);
    }

    let existing = read_claudemd(project_path)?;
    let info = inspect_project(&root)?;
    let improved = improve_markdown(&existing, &root, &info);

    Ok(GeneratedClaudeMd {
        project_path: root.display().to_string(),
        language: info.language,
        framework: info.framework,
        key_dirs: info.key_dirs,
        content: improved,
    })
}

#[derive(Debug, Clone)]
struct ProjectInfo {
    language: String,
    framework: Option<String>,
    key_dirs: Vec<String>,
    build_cmd: String,
    test_cmd: String,
    run_cmd: String,
    lint_cmd: String,
    install_cmd: String,
}

fn inspect_project(root: &Path) -> Result<ProjectInfo> {
    let mut ext_counts: HashMap<String, usize> = HashMap::new();
    let mut key_dirs: Vec<String> = Vec::new();

    collect_tree(root, root, 0, 4, &mut ext_counts, &mut key_dirs)?;

    key_dirs.sort();
    key_dirs.dedup();
    if key_dirs.len() > 8 {
        key_dirs.truncate(8);
    }

    let language = detect_language(&ext_counts);

    let pkg = read_package_json(root);
    let framework = detect_framework(root, pkg.as_ref(), &language);

    let (install_cmd, build_cmd, test_cmd, run_cmd, lint_cmd) =
        detect_commands(root, pkg.as_ref(), &language);

    Ok(ProjectInfo {
        language,
        framework,
        key_dirs,
        build_cmd,
        test_cmd,
        run_cmd,
        lint_cmd,
        install_cmd,
    })
}

fn collect_tree(
    base: &Path,
    dir: &Path,
    depth: usize,
    max_depth: usize,
    ext_counts: &mut HashMap<String, usize>,
    key_dirs: &mut Vec<String>,
) -> Result<()> {
    if depth > max_depth {
        return Ok(());
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();

        if path.is_dir() {
            if should_skip_dir(&name) {
                continue;
            }

            if depth <= 2 {
                let rel = path.strip_prefix(base).unwrap_or(&path);
                key_dirs.push(rel.display().to_string());
            }

            let _ = collect_tree(base, &path, depth + 1, max_depth, ext_counts, key_dirs);
            continue;
        }

        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            if !ext.is_empty() {
                *ext_counts.entry(ext.to_lowercase()).or_insert(0) += 1;
            }
        }
    }

    Ok(())
}

fn should_skip_dir(name: &str) -> bool {
    if name.starts_with('.') && name != ".github" {
        return true;
    }
    JUNK_DIRS.contains(&name)
}

fn detect_language(ext_counts: &HashMap<String, usize>) -> String {
    let candidates: &[(&str, &[&str])] = &[
        ("TypeScript", &["ts", "tsx"]),
        ("JavaScript", &["js", "jsx", "mjs", "cjs"]),
        ("Rust", &["rs"]),
        ("Python", &["py"]),
        ("Go", &["go"]),
        ("Swift", &["swift"]),
        ("Java", &["java"]),
        ("Kotlin", &["kt", "kts"]),
        ("C#", &["cs"]),
    ];

    let mut best = ("Unknown", 0usize);
    for (lang, exts) in candidates {
        let score: usize = exts
            .iter()
            .map(|ext| ext_counts.get(*ext).copied().unwrap_or(0))
            .sum();
        if score > best.1 {
            best = (lang, score);
        }
    }

    if best.1 == 0 {
        "General".to_string()
    } else {
        best.0.to_string()
    }
}

fn read_package_json(root: &Path) -> Option<Value> {
    let path = root.join("package.json");
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str::<Value>(&content).ok()
}

fn detect_framework(root: &Path, package_json: Option<&Value>, language: &str) -> Option<String> {
    if let Some(pkg) = package_json {
        let deps = merge_deps(pkg);
        if deps.contains_key("next") {
            return Some("Next.js".to_string());
        }
        if deps.contains_key("react") {
            return Some("React".to_string());
        }
        if deps.contains_key("vue") {
            return Some("Vue".to_string());
        }
        if deps.contains_key("svelte") {
            return Some("Svelte".to_string());
        }
        if deps.contains_key("nestjs") || deps.contains_key("@nestjs/core") {
            return Some("NestJS".to_string());
        }
        if deps.contains_key("express") {
            return Some("Express".to_string());
        }
    }

    if root.join("Cargo.toml").exists() {
        return Some("Cargo workspace".to_string());
    }
    if root.join("pyproject.toml").exists() {
        return Some("Python project".to_string());
    }
    if root.join("go.mod").exists() {
        return Some("Go module".to_string());
    }

    match language {
        "Rust" => Some("Cargo project".to_string()),
        "Python" => Some("Python project".to_string()),
        "Go" => Some("Go module".to_string()),
        _ => None,
    }
}

fn merge_deps(pkg: &Value) -> HashMap<String, String> {
    let mut out = HashMap::new();

    for key in ["dependencies", "devDependencies", "peerDependencies"] {
        if let Some(map) = pkg.get(key).and_then(|v| v.as_object()) {
            for (k, v) in map {
                out.insert(k.clone(), v.as_str().unwrap_or("*").to_string());
            }
        }
    }

    out
}

fn detect_commands(
    root: &Path,
    package_json: Option<&Value>,
    language: &str,
) -> (String, String, String, String, String) {
    if let Some(pkg) = package_json {
        let scripts = pkg
            .get("scripts")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();

        let has_pnpm = root.join("pnpm-lock.yaml").exists();
        let has_yarn = root.join("yarn.lock").exists();
        let has_bun = root.join("bun.lockb").exists() || root.join("bun.lock").exists();

        let runner = if has_pnpm {
            "pnpm"
        } else if has_yarn {
            "yarn"
        } else if has_bun {
            "bun"
        } else {
            "npm run"
        };

        let install_cmd = if has_pnpm {
            "pnpm install"
        } else if has_yarn {
            "yarn install"
        } else if has_bun {
            "bun install"
        } else {
            "npm install"
        }
        .to_string();

        let build_cmd = if scripts.contains_key("build") {
            format!("{} build", runner)
        } else {
            "npm run build".to_string()
        };

        let test_cmd = if scripts.contains_key("test") {
            format!("{} test", runner)
        } else {
            "npm run test".to_string()
        };

        let run_cmd = if scripts.contains_key("dev") {
            format!("{} dev", runner)
        } else if scripts.contains_key("start") {
            format!("{} start", runner)
        } else {
            "npm run dev".to_string()
        };

        let lint_cmd = if scripts.contains_key("lint") {
            format!("{} lint", runner)
        } else {
            "npm run lint".to_string()
        };

        return (install_cmd, build_cmd, test_cmd, run_cmd, lint_cmd);
    }

    if root.join("Cargo.toml").exists() || language == "Rust" {
        return (
            "cargo fetch".to_string(),
            "cargo build".to_string(),
            "cargo test".to_string(),
            "cargo run".to_string(),
            "cargo clippy --all-targets --all-features".to_string(),
        );
    }

    if root.join("pyproject.toml").exists()
        || root.join("requirements.txt").exists()
        || language == "Python"
    {
        return (
            "pip install -r requirements.txt".to_string(),
            "python -m build".to_string(),
            "pytest".to_string(),
            "python -m <entrypoint>".to_string(),
            "ruff check .".to_string(),
        );
    }

    if root.join("go.mod").exists() || language == "Go" {
        return (
            "go mod download".to_string(),
            "go build ./...".to_string(),
            "go test ./...".to_string(),
            "go run ./...".to_string(),
            "golangci-lint run".to_string(),
        );
    }

    (
        "<install dependencies>".to_string(),
        "<build command>".to_string(),
        "<test command>".to_string(),
        "<run command>".to_string(),
        "<lint command>".to_string(),
    )
}

fn render_markdown(root: &Path, info: &ProjectInfo) -> String {
    let key_dirs = if info.key_dirs.is_empty() {
        "- (add key directories here)".to_string()
    } else {
        info.key_dirs
            .iter()
            .map(|d| format!("- `{}`", d))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let framework = info
        .framework
        .clone()
        .unwrap_or_else(|| "(not detected)".to_string());

    format!(
        "# CLAUDE.md\n\n\
This file defines how Claude should operate in this repository.\n\n\
## Project Context\n\
- Repository root: `{}`\n\
- Primary language: {}\n\
- Framework/runtime: {}\n\
- Keep changes focused, minimal, and easy to review.\n\n\
## Build, Test, and Run\n\
Use these commands exactly before finalizing changes:\n\n\
```bash\n\
# Install dependencies\n\
{}\n\n\
# Build\n\
{}\n\n\
# Test\n\
{}\n\n\
# Lint\n\
{}\n\n\
# Run\n\
{}\n\
```\n\n\
## Repository Structure\n\
Key directories discovered during generation:\n\
{}\n\n\
## Coding Conventions\n\
- Prefer small, composable functions over large monolithic blocks.\n\
- Keep naming explicit and consistent with existing code style.\n\
- Avoid introducing new dependencies unless clearly justified.\n\
- Write tests for behavior changes and edge cases.\n\
- Preserve existing formatting/linting standards.\n\n\
## Workflow for Claude\n\
1. Read surrounding files before editing; do not guess interfaces.\n\
2. Propose or infer a minimal plan, then implement incrementally.\n\
3. Run build/test/lint commands relevant to the change.\n\
4. Summarize what changed, why, and any remaining risks.\n\n\
## Important Rules\n\
- Do not rewrite unrelated files.\n\
- Never remove user data or destructive commands without explicit approval.\n\
- Always call out assumptions when project intent is unclear.\n\
- Always include concrete file references in explanations.\n\n\
## Definition of Done\n\
- Build succeeds.\n\
- Tests relevant to the change pass.\n\
- New/changed behavior is documented in commit/summary notes.\n\
- No obvious regressions in existing flows.\n",
        root.display(),
        info.language,
        framework,
        info.install_cmd,
        info.build_cmd,
        info.test_cmd,
        info.lint_cmd,
        info.run_cmd,
        key_dirs,
    )
}

fn improve_markdown(existing: &str, root: &Path, info: &ProjectInfo) -> String {
    let mut out = existing.trim_end().to_string();
    if out.is_empty() {
        return render_markdown(root, info);
    }

    if !out.to_lowercase().contains("# claude.md") {
        out = format!("# CLAUDE.md\n\n{}", out.trim_start());
    }

    ensure_section(
        &mut out,
        "## Project Context",
        &format!(
            "## Project Context\n- Repository root: `{}`\n- Primary language: {}\n- Framework/runtime: {}\n- Keep changes focused, minimal, and easy to review.\n",
            root.display(),
            info.language,
            info.framework
                .clone()
                .unwrap_or_else(|| "(not detected)".to_string())
        ),
    );

    ensure_section(
        &mut out,
        "## Build, Test, and Run",
        &format!(
            "## Build, Test, and Run\nUse these commands exactly before finalizing changes:\n\n```bash\n# Install dependencies\n{}\n\n# Build\n{}\n\n# Test\n{}\n\n# Lint\n{}\n\n# Run\n{}\n```\n",
            info.install_cmd, info.build_cmd, info.test_cmd, info.lint_cmd, info.run_cmd
        ),
    );

    let key_dirs = if info.key_dirs.is_empty() {
        "- (add key directories here)".to_string()
    } else {
        info.key_dirs
            .iter()
            .map(|d| format!("- `{}`", d))
            .collect::<Vec<_>>()
            .join("\n")
    };
    ensure_section(
        &mut out,
        "## Repository Structure",
        &format!(
            "## Repository Structure\nKey directories discovered during generation:\n{}\n",
            key_dirs
        ),
    );

    ensure_section(
        &mut out,
        "## Coding Conventions",
        "## Coding Conventions\n- Prefer small, composable functions over large monolithic blocks.\n- Keep naming explicit and consistent with existing code style.\n- Avoid introducing new dependencies unless clearly justified.\n- Write tests for behavior changes and edge cases.\n- Preserve existing formatting/linting standards.\n",
    );

    ensure_section(
        &mut out,
        "## Workflow for Claude",
        "## Workflow for Claude\n1. Read surrounding files before editing; do not guess interfaces.\n2. Propose or infer a minimal plan, then implement incrementally.\n3. Run build/test/lint commands relevant to the change.\n4. Summarize what changed, why, and any remaining risks.\n",
    );

    ensure_section(
        &mut out,
        "## Important Rules",
        "## Important Rules\n- Do not rewrite unrelated files.\n- Never remove user data or destructive commands without explicit approval.\n- Always call out assumptions when project intent is unclear.\n- Always include concrete file references in explanations.\n",
    );

    ensure_section(
        &mut out,
        "## Definition of Done",
        "## Definition of Done\n- Build succeeds.\n- Tests relevant to the change pass.\n- New/changed behavior is documented in commit/summary notes.\n- No obvious regressions in existing flows.\n",
    );

    out.push('\n');
    out
}

fn ensure_section(content: &mut String, heading: &str, section_body: &str) {
    let has = content.to_lowercase().contains(&heading.to_lowercase());
    if has {
        return;
    }
    if !content.ends_with("\n\n") {
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push('\n');
    }
    content.push_str(section_body);
    if !content.ends_with('\n') {
        content.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let uniq = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let d = std::env::temp_dir().join(format!("claux-claudemd-{}", uniq));
        let _ = fs::create_dir_all(&d);
        d
    }

    #[test]
    fn generates_expected_sections() {
        let root = temp_dir();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname=\"x\"\n").unwrap();

        let generated = generate_for_project(root.to_str().unwrap()).unwrap();
        assert!(generated.content.contains("## Build, Test, and Run"));
        assert!(generated.content.contains("## Repository Structure"));
        assert!(generated.content.contains("## Workflow for Claude"));
        assert!(generated.content.contains("cargo build"));
    }

    #[test]
    fn improve_adds_missing_sections() {
        let root = temp_dir();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/main.rs"), "fn main() {}\\n").unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\\nname=\\\"x\\\"\\n").unwrap();
        fs::write(root.join("CLAUDE.md"), "# CLAUDE.md\\n\\nShort notes.\\n").unwrap();

        let improved = improve_for_project(root.to_str().unwrap()).unwrap();
        assert!(improved.content.contains("## Build, Test, and Run"));
        assert!(improved.content.contains("## Repository Structure"));
        assert!(improved.content.contains("## Definition of Done"));
    }
}
