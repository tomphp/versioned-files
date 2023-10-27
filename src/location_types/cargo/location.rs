use std::io::Read;
use std::process::{Child, Command, ExitStatus, Stdio};

use anyhow::anyhow;
use async_stream::stream;
use uuid::Uuid;

use crate::commands::update;
use crate::commands::update::Updater;

pub struct Location;

impl Location {
    pub const fn new() -> Self {
        Self
    }
}

impl Updater for Location {
    fn update(&self, version: String) -> update::EventStream {
        Box::pin(stream! {
            let toml_id = Uuid::new_v4();
            let lock_id = Uuid::new_v4();
            yield update::Event::Started(toml_id, "Cargo.toml".to_string());
            match update_cargo_version(&version) {
                Ok(()) => {
                  yield update::Event::Succeeded(toml_id);
                  yield update::Event::Started(lock_id, "Cargo.lock".to_string());
                  yield update::Event::Succeeded(lock_id);
                }
                Err(err) => {
                    yield update::Event::Failed(toml_id, err.to_string());
                }
            }
        })
    }
}

pub fn update_cargo_version(version: &str) -> anyhow::Result<()> {
    check_cargo_is_installed()?;
    install_cargo_edit()?;
    update_version(version)?;
    Ok(())
}

fn check_cargo_is_installed() -> anyhow::Result<()> {
    Command::new("cargo")
        .arg("--version")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| anyhow!("Failed to execute cargo, please make sure it is installed: {err}"))
        .and_then(wait_for_command)
        .and_then(returned_zero_or_error(|result| {
            anyhow!(
                "Failed to get cargo version (exit code: {}): {}",
                result.exit_status,
                result.stderr
            )
        }))
        .map(|_| ())
}

fn install_cargo_edit() -> anyhow::Result<()> {
    Command::new("cargo")
        .arg("install")
        .arg("cargo-edit")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| anyhow!("Failed install cargo-edit: {err}"))
        .and_then(wait_for_command)
        .and_then(returned_zero_or_error(|result| {
            anyhow!(
                "Failed to install cargo-edit (exit code: {}): {}",
                result.exit_status,
                result.stderr
            )
        }))
        .map(|_| ())
}

fn update_version(version: &str) -> anyhow::Result<()> {
    Command::new("cargo")
        .arg("set-version")
        .arg(version)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| anyhow!("Failed to update Cargo version: {err}"))
        .and_then(wait_for_command)
        .and_then(returned_zero_or_error(|result| {
            anyhow!(
                "Failed to set version (exit code: {}): {}",
                result.exit_status,
                result.stderr
            )
        }))
        .map(|_| ())
}

struct CommandResult {
    exit_status: ExitStatus,
    stderr: String,
}

fn wait_for_command(mut child: Child) -> anyhow::Result<CommandResult> {
    let exit_status = child.wait()?;

    let mut stderr = String::new();
    child
        .stderr
        .as_mut()
        .ok_or_else(|| anyhow!("failed to unwrap stderr"))?
        .read_to_string(&mut stderr)?;

    Ok(CommandResult {
        exit_status,
        stderr,
    })
}

fn returned_zero_or_error<E>(
    create_error: E,
) -> impl Fn(CommandResult) -> anyhow::Result<CommandResult>
where
    E: Fn(CommandResult) -> anyhow::Error,
{
    move |result| {
        if result.exit_status.success() {
            Ok(result)
        } else {
            Err(create_error(result))
        }
    }
}
