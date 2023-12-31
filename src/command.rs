use crate::formatter::plain::Plain;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Updates the version in all known locations
    Update {
        /// The new value to be used when representing a value
        #[arg()]
        version: String,
    },
}

impl Command {
    pub(crate) async fn execute(&self) -> Result<(), anyhow::Error> {
        match self {
            Self::Update { version } => {
                let formatter = Plain::new();
                crate::commands::update::Command::new(&formatter)
                    .execute(version)
                    .await?;
            }
        }

        Ok(())
    }
}
