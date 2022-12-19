mod db;
mod logic;
mod server;
mod util;

use std::backtrace::Backtrace;

pub use server::{handler, Config, Context};

#[derive(Debug)]
pub struct PublicError {
    message: String,
    #[allow(dead_code)]
    backtrace: Backtrace,
}

impl PublicError {
    #[track_caller]
    pub fn msg(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
            backtrace: Backtrace::capture(),
        }
    }
}

impl std::fmt::Display for PublicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for PublicError {}
