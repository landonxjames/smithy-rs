/*
 * Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
 * SPDX-License-Identifier: Apache-2.0
 */

use crate::{
    repo::Repo,
    tag::{previous_release_tag, release_tags},
};
use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};

mod command {
    mod audit;
    pub use audit::audit;

    mod patch;
    pub use patch::{patch, patch_with};
}

mod repo;
mod tag;
mod util;

#[derive(clap::Args, Clone)]
pub struct Audit {
    /// Don't `git fetch` before auditing.
    #[arg(long)]
    no_fetch: bool,
    /// Explicitly state the previous release's tag. Discovers it if not provided.
    #[arg(long)]
    previous_release_tag: Option<String>,
    /// Path to smithy-rs. Defaults to current working directory.
    #[arg(long)]
    smithy_rs_path: Option<Utf8PathBuf>,
    /// (For testing) Path to a fake crates.io index.
    #[arg(long)]
    fake_crates_io_index: Option<Utf8PathBuf>,
}

#[derive(clap::Args, Clone)]
pub struct PreviousReleaseTag {
    /// Path to smithy-rs. Defaults to current working directory.
    #[arg(long)]
    smithy_rs_path: Option<Utf8PathBuf>,
}

#[derive(clap::Args, Clone)]
pub struct PatchRuntime {
    /// Path to aws-sdk-rust.
    #[arg(long)]
    sdk_path: Utf8PathBuf,
    /// Path to smithy-rs. Defaults to current working directory.
    #[arg(long)]
    smithy_rs_path: Option<Utf8PathBuf>,
    /// Explicitly state the previous release's tag. Discovers it if not provided.
    #[arg(long)]
    previous_release_tag: Option<String>,
    /// Version number for stable crates.
    #[arg(long)]
    stable_crate_version: String,
    /// Version number for unstable crates.
    #[arg(long)]
    unstable_crate_version: String,
}

#[derive(clap::Args, Clone)]
pub struct PatchRuntimeWith {
    /// Path to aws-sdk-rust.
    #[arg(long)]
    sdk_path: Utf8PathBuf,
    /// Path to runtime crates to patch in.
    ///
    /// Note: this doesn't need to be a complete set of runtime crates. It will
    /// only patch the crates included in the provided path.
    #[arg(long)]
    runtime_crate_path: Utf8PathBuf,
    /// Explicitly state the previous release's tag. Discovers it if not provided.
    #[arg(long)]
    previous_release_tag: Option<String>,
}

#[derive(clap::Parser, Clone)]
#[clap(author, version, about)]
enum Command {
    /// Audit the runtime crate versions in the smithy-rs repo at HEAD
    ///
    /// Requires a full clone of smithy-rs. Will not work against shallow clones.
    ///
    /// This audits that any runtime crate that has been changed since the last
    /// release has been version bumped. It's not smart enough to know if the version
    /// bump is correct in semver terms, but verifies that there was at least a
    /// bump. A human will still need to verify the semver correctness of that bump.
    Audit(Audit),

    /// Outputs the previous release tag for the revision at HEAD.
    PreviousReleaseTag(PreviousReleaseTag),

    /// Patch a previous SDK release with the latest to-be-released runtime crates.
    ///
    /// This will generate a runtime with the given smithy-rs repo.
    PatchRuntime(PatchRuntime),

    /// Patch a previous SDK release with a given runtime.
    ///
    /// This will use an existing runtime at the path provided. For example,
    /// if you want to try a runtime from a GitHub Actions workflow.
    PatchRuntimeWith(PatchRuntimeWith),
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let command = Command::parse();
    match command {
        Command::Audit(args) => command::audit(args),
        Command::PreviousReleaseTag(args) => {
            let repo = Repo::new(args.smithy_rs_path.as_deref())?;
            let tags = release_tags(&repo)?;
            let tag = previous_release_tag(&repo, &tags, None)?;
            println!("{tag}");
            Ok(())
        }
        Command::PatchRuntime(args) => command::patch(args),
        Command::PatchRuntimeWith(args) => command::patch_with(args),
    }
}
