use clap::Parser;
use thiserror::Error;

use super::types::{CliArgs, FromArgs, FromArgsError, QueryTarget, ResolvedFromCommand};
use crate::infrastructure::cli::CliCommand;
use crate::ux_model::intent::{
    UserCommandContext, UserCommandDoctor, UserCommandFrom, UserCommandGc, UserCommandInit,
    UserCommandInteractive, UserWorkspace,
};

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

pub enum UserRequest {
    From(crate::ux_model::intent::UserRequest<UserCommandFrom>),
    Gc(crate::ux_model::intent::UserRequest<UserCommandGc>),
    Doctor(crate::ux_model::intent::UserRequest<UserCommandDoctor>),
    Init(crate::ux_model::intent::UserRequest<UserCommandInit>),
    Interactive(crate::ux_model::intent::UserRequest<UserCommandInteractive>),
}

pub fn resolve_intent(args: CliArgs) -> Result<UserRequest, IntentResolutionError> {
    let context = UserCommandContext {
        user_workspace: UserWorkspace::Pwd,
    };

    match args.command {
        CliCommand::From(from_args) => {
            let resolved = resolve_from_args(from_args)
                .map_err(|source| IntentResolutionError::InvalidFromCommand { source })?;
            let cmd = match resolved.target {
                QueryTarget::Symbol(symbol) => UserCommandFrom::symbol(symbol, resolved.depth),
                QueryTarget::File(file) => UserCommandFrom::file(file, resolved.depth),
                QueryTarget::Module(module) => UserCommandFrom::module(module, resolved.depth),
                QueryTarget::PublicExports => UserCommandFrom::public_exports(resolved.depth),
            };
            Ok(UserRequest::From(crate::ux_model::intent::UserRequest::terminal(cmd, context)))
        }
        CliCommand::Gc => {
            Ok(UserRequest::Gc(crate::ux_model::intent::UserRequest::terminal(UserCommandGc, context)))
        }
        CliCommand::Doctor => {
            Ok(UserRequest::Doctor(crate::ux_model::intent::UserRequest::terminal(UserCommandDoctor, context)))
        }
        CliCommand::Init => {
            Ok(UserRequest::Init(crate::ux_model::intent::UserRequest::terminal(UserCommandInit, context)))
        }
        CliCommand::Interactive => {
            Ok(UserRequest::Interactive(crate::ux_model::intent::UserRequest::interactive(UserCommandInteractive, context)))
        }
    }
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
