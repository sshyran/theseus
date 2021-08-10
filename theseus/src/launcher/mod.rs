//! Utilities for launching minecraft

use crate::launcher::auth::provider::Credentials;
use std::path::Path;
use std::process::{Command, Stdio};
use thiserror::Error;

pub mod args;
pub mod auth;
pub mod download;
pub mod java;
pub mod meta;
pub mod rules;
pub mod profile;

#[derive(Error, Debug)]
pub enum LauncherError {
    #[error("Failed to violate file checksum at url {url} with hash {hash} after {tries} tries")]
    ChecksumFailure {
        hash: String,
        url: String,
        tries: u32,
    },
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Error while managing asynchronous tasks")]
    TaskError(#[from] tokio::task::JoinError),
    #[error("Error while reading/writing to the disk")]
    IoError(#[from] std::io::Error),
    #[error("Error while spawning child process {process}")]
    ProcessError {
        inner: std::io::Error,
        process: String,
    },
    #[error("Error while deserializing JSON")]
    SerdeError(#[from] serde_json::Error),
    #[error("Unable to fetch {item}")]
    FetchError { inner: reqwest::Error, item: String },
    #[error("{0}")]
    ParseError(String),
}

pub struct LaunchOptions<'a> {
    pub root_dir: &'a Path,
    pub assets_dir: &'a Path,
    pub legacy_assets_dir: &'a Path,
    pub game_dir: &'a Path,
    pub natives_dir: &'a Path,
    pub libraries_dir: &'a Path,
    pub client_dir: &'a Path,
}

pub async fn launch_minecraft(
    options: &LaunchOptions<'_>,
    version_name: &str,
    credentials: &Credentials,
) -> Result<(), LauncherError> {
    let manifest = meta::fetch_version_manifest().await.unwrap();

    let version = download::download_version_info(
        &options.client_dir,
        manifest
            .versions
            .iter()
            .find(|x| x.id == version_name)
            .ok_or_else(|| {
                LauncherError::InvalidInput(format!("Version {} does not exist", version_name))
            })?,
    )
    .await?;

    download_minecraft(options, &version).await?;

    let arguments = version.arguments.unwrap();

    let mut child = Command::new("java")
        .args(args::get_jvm_arguments(
            arguments
                .get(&meta::ArgumentType::Jvm)
                .map(|x| x.as_slice()),
            &*options.natives_dir.join(&version.id),
            &*args::get_class_paths(
                options.libraries_dir,
                version.libraries.as_slice(),
                &*options.client_dir
                    .join(&version.id)
                    .join(format!("{}.jar", &version.id)),
            )?,
        )?)
        .arg(version.main_class)
        .args(args::get_minecraft_arguments(
            arguments
                .get(&meta::ArgumentType::Game)
                .map(|x| x.as_slice()),
            version.minecraft_arguments.as_deref(),
            credentials,
            &*version.id,
            &version.asset_index.id,
            options.game_dir,
            options.assets_dir,
            &version.type_,
        )?)
        .current_dir(&options.root_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|err| LauncherError::ProcessError {
            inner: err,
            process: "minecraft".to_string(),
        })?;

    child.wait().map_err(|err| LauncherError::ProcessError {
        inner: err,
        process: "minecraft".to_string(),
    })?;

    Ok(())
}

pub async fn download_minecraft(
    options: &LaunchOptions<'_>,
    version: &meta::VersionInfo,
) -> Result<(), LauncherError> {
    let assets_index = download::download_assets_index(options.assets_dir, &version).await?;

    let (a, b, c) = futures::future::join3(
        download::download_client(options.client_dir, &version),
        download::download_assets(
            options.assets_dir,
            if version.assets == "legacy" {
                Some(options.legacy_assets_dir)
            } else {
                None
            },
            &assets_index,
        ),
        download::download_libraries(
            options.libraries_dir,
            &*options.natives_dir.join(&version.id),
            version.libraries.as_slice(),
        ),
    )
    .await;

    a?;
    b?;
    c?;

    Ok(())
}
