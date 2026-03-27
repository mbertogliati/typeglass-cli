use std::future::Future;

use super::action::UserAction;
use super::error::UseCaseError;

pub trait UseCase {
    type Input;
    type Output;

    const ACTION: UserAction;
}

pub trait UseCaseExecutor<U: UseCase> {
    type ExecuteFuture<'a>: Future<Output = Result<U::Output, UseCaseError>> + Send + 'a
    where
        Self: 'a;

    fn execute<'a>(&'a self, input: U::Input) -> Self::ExecuteFuture<'a>;
}
