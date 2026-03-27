mod cli;
mod domain;
mod ux_model;

fn main() {
    env_logger::init();

    let args = cli::parse_cli();
    let request = match cli::resolve_intent(args) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    let intent_contract = ux_model::UserIntentContract::from_request(&request);
    let outcome_contract = ux_model::outcome_contract_for_command(intent_contract.command);

    match serde_json::to_string_pretty(&serde_json::json!({
        "intent_request": request,
        "intent_contract": intent_contract,
        "outcome_contract": outcome_contract,
    })) {
        Ok(payload) => println!("{payload}"),
        Err(error) => {
            eprintln!("Failed to serialize CLI outcome. Reason: {error}");
            std::process::exit(1);
        }
    }
}
