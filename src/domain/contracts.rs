pub trait UserMessage {
    fn user_message(&self) -> String;
}

pub trait MachineCode {
    fn code(&self) -> &'static str;
}
