use clap::Parser;
use thiserror::Error;

use super::types::{CliArgs, FromArgs, FromArgsError, QueryTarget, ResolvedFromCommand};
use crate::cli::CliCommand;
use crate::ux_model::{FromTarget, UserCommand, UserMode, UserRequest};

pub fn parse_cli() -> CliArgs {
    CliArgs::parse()
}

pub fn resolve_from_args(args: FromArgs) -> Result<ResolvedFromCommand, FromArgsError> {
    let mut targets = Vec::new();

    if let Some(symbol) = args.symbol {
        targets.push(QueryTarget::Symbol(symbol));
    }
    if let Some(file) = args.file {
        targets.push(QueryTarget::File(file));
    }
    if let Some(module) = args.module {
        targets.push(QueryTarget::Module(module));
    }
    if args.public_exports {
        targets.push(QueryTarget::PublicExports);
    }

    match targets.len() {
        0 => Err(FromArgsError::MissingTarget),
        1 => Ok(ResolvedFromCommand {
            target: targets.remove(0),
            depth: args.depth,
        }),
        _ => Err(FromArgsError::MultipleTargets),
    }
}

#[derive(Debug, Error)]
pub enum IntentResolutionError {
    #[error("Cannot resolve `from` command. Reason: {source}")]
    InvalidFromCommand { source: FromArgsError },
}

pub fn resolve_intent(args: CliArgs) -> Result<UserRequest, IntentResolutionError> {
    let mode = UserMode::Terminal;

    let command = match args.command {
        CliCommand::From(from_args) => {
            let resolved = resolve_from_args(from_args)
                .map_err(|source| IntentResolutionError::InvalidFromCommand { source })?;
            let target = match resolved.target {
                QueryTarget::Symbol(symbol) => FromTarget::Symbol(symbol),
                QueryTarget::File(file) => FromTarget::File(file),
                QueryTarget::Module(module) => FromTarget::Module(module),
                QueryTarget::PublicExports => FromTarget::PublicExports,
            };
            UserCommand::From {
                target,
                depth: resolved.depth,
            }
        }
        CliCommand::Gc => UserCommand::Gc,
        CliCommand::Doctor => UserCommand::Doctor,
        CliCommand::Init => UserCommand::Init,
        CliCommand::Interactive => {
            return Ok(UserRequest {
                mode: UserMode::Interactive,
                command: UserCommand::Interactive,
            });
        }
    };

    Ok(UserRequest { mode, command })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_from_args_requires_target() {
        let args = FromArgs {
            symbol: None,
            file: None,
            module: None,
            public_exports: false,
            depth: Some(2),
        };

        let result = resolve_from_args(args);
        assert!(matches!(result, Err(FromArgsError::MissingTarget)));
    }

    #[test]
    fn resolve_from_args_rejects_multiple_targets() {
        let args = FromArgs {
            symbol: Some("OrderService".to_string()),
            file: None,
            module: None,
            public_exports: true,
            depth: None,
        };

        let result = resolve_from_args(args);
        assert!(matches!(result, Err(FromArgsError::MultipleTargets)));
    }

    #[test]
    fn resolve_from_args_accepts_single_target() {
        let args = FromArgs {
            symbol: Some("OrderService".to_string()),
            file: None,
            module: None,
            public_exports: false,
            depth: Some(1),
        };

        let result = resolve_from_args(args);
        assert!(matches!(
            result,
            Ok(ResolvedFromCommand {
                target: QueryTarget::Symbol(_),
                depth: Some(1)
            })
        ));
    }
}
