use crate::error::Error;
use clap::{Args, CommandFactory, Parser, Subcommand};

use crate::commands;

/// Minecraft Java 26.1.2 structure container and loot finder.
#[derive(Debug, Parser)]
#[command(disable_version_flag = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List possible structure chunks without full verification.
    Candidates(SearchArgs),
    /// Verify structures and list their block containers.
    Chests(SearchArgs),
    /// Find containers that generate the requested item.
    Find(FindArgs),
    /// Replay one supported loot table.
    Loot(LootArgs),
    /// Calculate a seed for supported shortcut structures.
    ContainerSeed(ContainerSeedArgs),
    /// Show defaults, supported structures, and loot tables.
    Explain(ExplainArgs),
}

#[derive(Debug, Args)]
pub struct CommonArgs {
    /// Minecraft version to target; only 26.1.2 is supported.
    #[arg(long, value_name = "VERSION", default_value = "26.1.2")]
    pub version: String,
    /// Emit machine-readable JSON.
    #[arg(long)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct SearchArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    /// World seed.
    #[arg(long, allow_hyphen_values = true)]
    pub seed: i64,
    /// Structure to search for.
    #[arg(long, default_value = "ancient_city")]
    pub structure: String,
    /// Center of the search area on the X axis.
    #[arg(long, allow_hyphen_values = true, default_value_t = 0)]
    pub center_x: i32,
    /// Center of the search area on the Z axis.
    #[arg(long, allow_hyphen_values = true, default_value_t = 0)]
    pub center_z: i32,
    /// Search radius in blocks.
    #[arg(long, allow_hyphen_values = true, value_parser = clap::value_parser!(i32).range(0..))]
    pub radius: Option<i32>,
    /// Maximum number of results to display.
    #[arg(long, allow_hyphen_values = true, value_parser = clap::value_parser!(i32).range(0..))]
    pub limit: Option<i32>,
}

#[derive(Debug, Args)]
pub struct FindArgs {
    #[command(flatten)]
    pub search: SearchArgs,
    /// Item to search for.
    #[arg(long)]
    pub item: Option<String>,
}

#[derive(Debug, Args)]
pub struct LootArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    /// Loot table seed.
    #[arg(long, allow_hyphen_values = true)]
    pub loot_seed: i64,
    /// Loot table to replay.
    #[arg(long, default_value = "minecraft:chests/ancient_city")]
    pub table: String,
}

#[derive(Debug, Args)]
pub struct ContainerSeedArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    /// Structure to calculate the seed for.
    #[arg(long, default_value = "ancient_city")]
    pub structure: String,
    /// World seed.
    #[arg(long, allow_hyphen_values = true)]
    pub seed: i64,
    /// Structure or decoration chunk X coordinate.
    #[arg(long, allow_hyphen_values = true, default_value_t = 0)]
    pub chunk_x: i32,
    /// Structure or decoration chunk Z coordinate.
    #[arg(long, allow_hyphen_values = true, default_value_t = 0)]
    pub chunk_z: i32,
    /// Structure index override.
    #[arg(long, allow_hyphen_values = true)]
    pub structure_index: Option<i32>,
    /// Decoration step override.
    #[arg(long, allow_hyphen_values = true)]
    pub step: Option<i32>,
    /// Container ordinal within the chunk.
    #[arg(long, allow_hyphen_values = true, default_value_t = 0)]
    pub ordinal: i32,
}

#[derive(Debug, Args)]
pub struct ExplainArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    /// Structure to describe in detail.
    #[arg(long)]
    pub structure: Option<String>,
}

pub fn run(arguments: impl IntoIterator<Item = String>) -> Result<u8, Error> {
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    if arguments.is_empty() {
        print_help();
        return Ok(0);
    }
    let cli =
        match Cli::try_parse_from(std::iter::once("mc-loot-finder".to_owned()).chain(arguments)) {
            Ok(cli) => cli,
            Err(error) => {
                return match error.kind() {
                    clap::error::ErrorKind::DisplayHelp
                    | clap::error::ErrorKind::DisplayVersion => {
                        let _ = error.print();
                        Ok(error.exit_code() as u8)
                    }
                    clap::error::ErrorKind::MissingSubcommand => {
                        print_help();
                        Ok(0)
                    }
                    _ => {
                        let _ = error.print();
                        Ok(2)
                    }
                };
            }
        };
    match cli.command {
        Command::Candidates(args) => commands::candidates::run(args),
        Command::Chests(args) => commands::chests::run(args),
        Command::Find(args) => commands::find::run(args),
        Command::Loot(args) => commands::loot::run(args),
        Command::ContainerSeed(args) => commands::container_seed::run(args),
        Command::Explain(args) => commands::explain::run(args),
    }
}

pub(crate) fn require_version(version: &str) -> Result<(), Error> {
    if version == "26.1.2" {
        Ok(())
    } else {
        Err(Error::Usage(format!(
            "unsupported Minecraft version: {version}; supported: 26.1.2"
        )))
    }
}

pub(crate) fn require_identifier(value: &str, option: &str) -> Result<(), Error> {
    let Some((namespace, path)) = value.split_once(':') else {
        return Err(Error::Usage(format!(
            "{option} must be a namespaced Minecraft id"
        )));
    };
    let valid_namespace = !namespace.is_empty()
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        });
    let valid_path = !path.is_empty()
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_./-".contains(&byte)
        });
    if valid_namespace && valid_path {
        Ok(())
    } else {
        Err(Error::Usage(format!(
            "{option} must be a namespaced Minecraft id"
        )))
    }
}

/// Print the clap-generated help for the top-level command.
fn print_help() {
    let mut command = Cli::command();
    let _ = command.print_help();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_find_with_negative_seed() {
        let cli = Cli::try_parse_from([
            "mc-loot-finder",
            "find",
            "--seed",
            "-1",
            "--item",
            "minecraft:bedrock",
            "--json",
        ])
        .unwrap();
        let Command::Find(args) = cli.command else {
            panic!("expected find command");
        };
        assert_eq!(args.search.seed, -1);
        assert_eq!(args.item.as_deref(), Some("minecraft:bedrock"));
        assert!(args.search.common.json);
    }

    #[test]
    fn applies_per_command_defaults() {
        let cli = Cli::try_parse_from(["mc-loot-finder", "chests", "--seed", "0"]).unwrap();
        let Command::Chests(args) = cli.command else {
            panic!("expected chests command");
        };
        assert_eq!(args.structure, "ancient_city");
        assert_eq!(args.radius, None);
        assert_eq!(args.limit, None);
        assert_eq!(args.center_x, 0);
        assert_eq!(args.common.version, "26.1.2");
    }

    #[test]
    fn rejects_missing_required_seed() {
        assert!(Cli::try_parse_from(["mc-loot-finder", "find"]).is_err());
        assert!(Cli::try_parse_from(["mc-loot-finder", "loot"]).is_err());
        assert!(Cli::try_parse_from(["mc-loot-finder", "container-seed"]).is_err());
    }

    #[test]
    fn rejects_unknown_option() {
        assert!(
            Cli::try_parse_from(["mc-loot-finder", "find", "--seed", "0", "--bogus", "1",])
                .is_err()
        );
    }

    #[test]
    fn rejects_negative_limit() {
        assert!(
            Cli::try_parse_from(["mc-loot-finder", "find", "--seed", "0", "--limit", "-1",])
                .is_err()
        );
    }

    #[test]
    fn help_lists_all_commands() {
        let mut command = Cli::command();
        let help = command.render_help().to_string();
        for command in [
            "candidates",
            "chests",
            "find",
            "loot",
            "container-seed",
            "explain",
        ] {
            assert!(help.contains(command), "help must mention {command}");
        }
    }
}
