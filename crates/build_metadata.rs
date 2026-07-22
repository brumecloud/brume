use std::{env, fs, io, path::Path, process::Command};

pub struct BuildMetadata {
    commit_sha: String,
    commit_title: String,
    commit_message: String,
}

pub fn collect(repository: &Path) -> BuildMetadata {
    for variable in [
        "BRUME_BUILD_COMMIT",
        "BRUME_BUILD_COMMIT_TITLE",
        "BRUME_BUILD_COMMIT_MESSAGE",
        "RAILWAY_GIT_COMMIT_SHA",
        "RAILWAY_GIT_COMMIT_MESSAGE",
    ] {
        println!("cargo:rerun-if-env-changed={variable}");
    }
    track_git_head(repository);

    let railway_message = env::var("RAILWAY_GIT_COMMIT_MESSAGE").ok();
    let (railway_title, railway_body) = railway_message
        .as_deref()
        .map(split_commit_message)
        .unwrap_or_default();

    BuildMetadata {
        commit_sha: non_empty_env("BRUME_BUILD_COMMIT")
            .or_else(|| non_empty_env("RAILWAY_GIT_COMMIT_SHA"))
            .or_else(|| git(repository, &["rev-parse", "HEAD"]))
            .unwrap_or_else(|| "unknown".to_owned()),
        commit_title: non_empty_env("BRUME_BUILD_COMMIT_TITLE")
            .or(railway_title)
            .or_else(|| git(repository, &["log", "-1", "--format=%s"]))
            .unwrap_or_else(|| "unknown".to_owned()),
        commit_message: env::var("BRUME_BUILD_COMMIT_MESSAGE")
            .ok()
            .or(railway_body)
            .or_else(|| git(repository, &["log", "-1", "--format=%b"]))
            .unwrap_or_default(),
    }
}

pub fn write_rust_source(output: &Path, metadata: &BuildMetadata) -> io::Result<()> {
    fs::write(
        output.join("build_metadata.rs"),
        format!(
            "pub const COMMIT_SHA: &str = {:?};\n\
             pub const COMMIT_TITLE: &str = {:?};\n\
             pub const COMMIT_MESSAGE: &str = {:?};\n",
            metadata.commit_sha, metadata.commit_title, metadata.commit_message
        ),
    )
}

fn non_empty_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.is_empty())
}

fn split_commit_message(message: &str) -> (Option<String>, Option<String>) {
    let (title, body) = message
        .split_once('\n')
        .map_or((message, ""), |(title, body)| (title, body));
    let title = title.trim_end_matches('\r');
    let body = body.trim_end();
    (
        (!title.is_empty()).then(|| title.to_owned()),
        Some(body.to_owned()),
    )
}

fn track_git_head(repository: &Path) {
    let Some(head) = git(repository, &["rev-parse", "--git-path", "HEAD"]) else {
        return;
    };
    println!("cargo:rerun-if-changed={}", repository.join(head).display());

    let Some(reference) = git(repository, &["symbolic-ref", "-q", "HEAD"]) else {
        return;
    };
    if let Some(reference_path) = git(repository, &["rev-parse", "--git-path", &reference]) {
        println!(
            "cargo:rerun-if-changed={}",
            repository.join(reference_path).display()
        );
    }
}

fn git(repository: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| {
            String::from_utf8_lossy(&output.stdout)
                .trim_end()
                .to_owned()
        })
        .filter(|value| !value.is_empty())
}
