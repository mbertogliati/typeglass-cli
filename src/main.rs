mod application;
mod cli;
mod domain;
mod ux_model;

use crate::ux_model::intent::UserCommandType;
use crate::ux_model::UserRequest;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = cli::parse_cli();
    let request = match cli::resolve_intent(args) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    let command = request.into_command();
    let service = application::ApplicationService::new(application::UnwiredAdapters);
    
    match command.command_type {
        UserCommandType::From { .. } => {
            let response = service.execute::<application::FromAction>(command).await;
            print_response(response);
        }
        UserCommandType::Gc => {
            let response = service.execute::<application::GcAction>(command).await;
            print_response(response);
        }
        _ => {
            println!("Command not yet implemented in main dispatcher");
        }
    }
}

fn print_response<S, P, F>(response: ux_model::result::UserResult<S, P, F>)
where
    S: ux_model::result::SuccessUserExpectations,
    P: ux_model::result::PartialSuccessUserExpectations,
    F: ux_model::result::FailureUserExpectations,
{
    match serde_json::to_string_pretty(&serde_json::json!({
        "status": format!("{:?}", response.status()),
        "summary": response.summary().0,
        "details": format!("{:#?}", response),
    })) {
        Ok(payload) => println!("{payload}"),
        Err(error) => {
            eprintln!("Failed to serialize CLI outcome. Reason: {error}");
            std::process::exit(1);
        }
    }
}
