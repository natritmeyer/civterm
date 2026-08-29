#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    message: String,
}

impl Event {
    pub fn new(message: impl Into<String>) -> Self {
        Event {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}
