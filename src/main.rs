use typeglass_cli::application;
use typeglass_cli::infrastructure::cli;
use typeglass_cli::ux_model;
use typeglass_cli::infrastructure::cli::UserRequest;
use clap::Parser;

#[tokio::main]
async fn main() {
    // Parse CLI args first
    let args = cli::parse_cli();
    
    // Configure logging based on --debug flag
    if args.debug {
        std::env::set_var("RUST_LOG", "debug");
    } else {
        std::env::set_var("RUST_LOG", "warn");
    }
    env_logger::init();
    
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
        UserRequest::From(req, json, _) => {
            let (action, context) = req.into_parts();
            let response = service.execute(action, context).await;
            print_response(response, json);
        }
        UserRequest::Gc(req, json, _) => {
            let (action, context) = req.into_parts();
            let response = service.execute(action, context).await;
            print_response(response, json);
        }
        UserRequest::Doctor(req, json, _) => {
            let (action, context) = req.into_parts();
            let response = service.execute(action, context).await;
            print_response(response, json);
        }
        UserRequest::Init(req, json, _) => {
            let (action, context) = req.into_parts();
            let response = service.execute(action, context).await;
            print_response(response, json);
        }
        UserRequest::Interactive(req, json, _) => {
            let (action, context) = req.into_parts();
            let response = service.execute(action, context).await;
            print_response(response, json);
        }
    }
}

fn print_response<S, P, F>(response: ux_model::result::UserResult<S, P, F>, json: bool)
where
    S: ux_model::result::SuccessUserExpectations,
    P: ux_model::result::PartialSuccessUserExpectations,
    F: ux_model::result::FailureUserExpectations,
{
    use ux_model::result::UserResult;
    
    if json {
        // JSON output (original behavior)
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
    } else {
        // Human-readable output (default)
        match response {
            UserResult::Success(_) => {
                println!("✅ {}", response.summary().0);
            }
            UserResult::Partial(_) => {
                println!("⚠️  {}", response.summary().0);
            }
            UserResult::Failure(_) => {
                eprintln!("❌ {}", response.summary().0);
                std::process::exit(1);
            }
        }
    }
}
