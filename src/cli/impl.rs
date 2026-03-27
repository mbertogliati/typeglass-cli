use clap::Parser;

use super::types::{CliArgs, FromArgs, FromArgsError, QueryTarget, ResolvedFromCommand};

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
