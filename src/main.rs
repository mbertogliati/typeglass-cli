use typeglass_cli::application;
use typeglass_cli::infrastructure::cli;
use typeglass_cli::ux_model;
use typeglass_cli::infrastructure::cli::UserRequest;
use clap::Parser;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = cli::parse_cli();
    let request = match cli::resolve_intent(args) {
        Ok(request) => request,
        Err(cli::IntentResolutionError::ShowHelp) => {
            // Print help and exit gracefully
            cli::CliArgs::parse_from(&["typeglass", "--help"]);
            unreachable!("clap will print help and exit");
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    let service = application::ApplicationService::new(application::UnwiredAdapters);
    
    match request {
        UserRequest::From(req) => {
            let (action, context) = req.into_parts();
            let response = service.execute(action, context).await;
            print_response(response);
        }
        UserRequest::Gc(req) => {
            let (action, context) = req.into_parts();
            let response = service.execute(action, context).await;
            print_response(response);
        }
        UserRequest::Doctor(req) => {
            let (action, context) = req.into_parts();
            let response = service.execute(action, context).await;
            print_response(response);
        }
        UserRequest::Init(req) => {
            let (action, context) = req.into_parts();
            let response = service.execute(action, context).await;
            print_response(response);
        }
        UserRequest::Interactive(req) => {
            let (action, context) = req.into_parts();
            let response = service.execute(action, context).await;
            print_response(response);
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
