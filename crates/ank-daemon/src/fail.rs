//! A refusal, rendered the way the CLI renders one (§4).
//!
//! The daemon dispatches no verb and answers no question, so it has no output
//! contract of its own. What it does have is a person reading its stderr when
//! it will not start, and that person already knows one shape of refusal:
//! `error[9]:` with the code §4 gives the cause, and an arrow naming the exact
//! thing that resolves it. A second shape would be a second thing to learn for
//! no gain, so this is the first one and nothing more.

use ank_contract::ExitCode;
use std::io::Write;

pub type Result<T> = std::result::Result<T, Fail>;

#[derive(Debug)]
pub struct Fail {
    pub code: ExitCode,
    pub message: String,
    pub hint: Option<String>,
}

impl Fail {
    pub fn new(code: ExitCode, message: impl Into<String>) -> Fail {
        Fail {
            code,
            message: message.into(),
            hint: None,
        }
    }

    /// The next command, or the exact correction. Every refusal in this crate
    /// carries one, on the rule the CLI holds itself to: a refusal that names
    /// no way forward is a refusal the reader has to guess past.
    pub fn with_hint(mut self, hint: impl Into<String>) -> Fail {
        self.hint = Some(hint.into());
        self
    }

    pub fn report(&self, err: &mut dyn Write) {
        let _ = writeln!(err, "error[{}]: {}", self.code, self.message);
        if let Some(hint) = &self.hint {
            let _ = writeln!(err, "  -> {hint}");
        }
    }
}
