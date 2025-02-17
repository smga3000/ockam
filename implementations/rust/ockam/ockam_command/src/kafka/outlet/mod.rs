pub(crate) mod create;

use self::create::CreateCommand;
use crate::{Command, CommandGlobalOpts};
use clap::{command, Args, Subcommand};

/// Manage Kafka Outlets
#[derive(Clone, Debug, Args)]
#[command(arg_required_else_help = true, subcommand_required = true)]
pub struct KafkaOutletCommand {
    #[command(subcommand)]
    pub(crate) subcommand: KafkaOutletSubcommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum KafkaOutletSubcommand {
    Create(CreateCommand),
}

impl KafkaOutletCommand {
    pub fn run(self, opts: CommandGlobalOpts) -> miette::Result<()> {
        match self.subcommand {
            KafkaOutletSubcommand::Create(c) => c.run(opts),
        }
    }

    pub fn name(&self) -> String {
        match &self.subcommand {
            KafkaOutletSubcommand::Create(c) => c.name(),
        }
        .to_string()
    }
}
