mod application;
mod cli;
mod domain;
mod ux_model;

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

    let response = application::ApplicationService::new(application::UnwiredAdapters)
        .execute(request)
        .await;
    let expectations = response.user_expectations();
    let user_result = response.user_result();

    match serde_json::to_string_pretty(&serde_json::json!({
        "request": format!("{:#?}", response.request),
        "intent_contract": format!("{:#?}", response.intent_contract),
        "expectations": format!("{:#?}", expectations),
        "command_result": format!("{:#?}", response.result),
        "user_result": format!("{:#?}", user_result),
    })) {
        Ok(payload) => println!("{payload}"),
        Err(error) => {
            eprintln!("Failed to serialize CLI outcome. Reason: {error}");
            std::process::exit(1);
        }
    }
}
