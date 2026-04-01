use typeglass_cli::application;
use typeglass_cli::domain::language::Language;
use typeglass_cli::infrastructure::cli;
use typeglass_cli::infrastructure::cli::{UserRequest, OutputFormat};
use typeglass_cli::infrastructure::adapters::WiredAdapters;
use typeglass_cli::ux_model;
use clap::Parser;
use std::path::Path;

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
            cli::CliArgs::parse_from(["typeglass", "--help"]);
            unreachable!("clap will print help and exit");
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    // Get workspace root and detect language
    let workspace_root = std::env::current_dir().expect("Failed to get current directory");
    let language = detect_language(&workspace_root);

    // Create service with REAL adapters (not UnwiredAdapters)
    let service = application::ApplicationService::new(
        WiredAdapters::new(workspace_root, language)
    );
    
    match request {
        UserRequest::From(req, format, _) => {
            let (action, context) = req.into_parts();
            let response = service.execute(action, context).await;
            print_from_response(response, format);
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

fn print_from_response<S, P, F>(response: ux_model::result::UserResult<S, P, F>, format: OutputFormat)
where
    S: ux_model::result::SuccessUserExpectations,
    P: ux_model::result::PartialSuccessUserExpectations,
    F: ux_model::result::FailureUserExpectations,
{
    
    
    match format {
        OutputFormat::Human => {
            print_response(response, false);
        }
        OutputFormat::Json => {
            print_response(response, true);
        }
        OutputFormat::Dot | OutputFormat::Mermaid => {
            // For now, just print human-readable
            // TODO: Extract graph from response and format it
            // This would require accessing the graph data from the response
            // which needs deeper integration
            eprintln!("⚠️  DOT and Mermaid output formats are not yet fully integrated.");
            eprintln!("    Falling back to human-readable output.");
            eprintln!("    To use these formats, pipe the JSON output to a separate tool:");
            eprintln!("    typeglass from --symbol MyType --format json | typeglass-format --dot");
            print_response(response, false);
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

/// Detect project language from workspace markers
fn detect_language(workspace_root: &Path) -> Language {
    if workspace_root.join("Cargo.toml").exists() {
        Language::Rust
    } else if workspace_root.join("package.json").exists() || workspace_root.join("tsconfig.json").exists() {
        Language::TypeScript
    } else if workspace_root.join("go.mod").exists() {
        Language::Go
    } else {
        // Default to Rust if detection fails
        eprintln!("Warning: Could not detect language, defaulting to Rust");
        Language::Rust
    }
}
