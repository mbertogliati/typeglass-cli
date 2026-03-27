use std::future::Future;
use crate::ux_model::intent::UserCommandContext;
use crate::application::adapters::ApplicationAdapters;
use crate::application::types::{ApplicationOutcome, CommandAction};

/// El servicio de aplicación que orquesta la ejecución de comandos.
pub struct ApplicationService<A: ApplicationAdapters> {
    pub(crate) adapters: A,
}

impl<A: ApplicationAdapters> ApplicationService<A> {
    pub fn new(adapters: A) -> Self {
        Self { adapters }
    }

    /// Método principal de ejecución. Es asíncrono, recibe una acción 
    /// y devuelve un resultado vinculado estáticamente a dicha acción.
    pub async fn execute<C>(&self, action: C, context: UserCommandContext) -> ApplicationOutcome<C>
    where
        C: CommandAction,
        Self: ActionExecutor<C>,
    {
        self.execute_action(action, context).await
    }
}

/// Trait para ejecutar una acción específica.
pub trait ActionExecutor<C: CommandAction> {
    fn execute_action(&self, action: C, context: UserCommandContext) -> impl Future<Output = ApplicationOutcome<C>> + Send;
}
