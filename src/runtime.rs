pub mod action;
pub mod args;
pub mod blackboard;
pub mod builder;
pub mod context;
pub mod forester;
pub mod rtree;

#[cfg(test)]
mod tests;

use crate::runtime::action::Tick;
use crate::tree::TreeError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type RtResult<T> = Result<T, RuntimeError>;
pub type RtOk = Result<(), RuntimeError>;
#[derive(Clone, Debug, PartialEq)]
pub enum TickResult {
    Success,
    Failure(String),
    Running,
}

impl TickResult {
    pub fn success() -> TickResult {
        TickResult::Success
    }
    pub fn failure_empty() -> TickResult {
        TickResult::Failure("".to_string())
    }
    pub fn failure(reason: String) -> TickResult {
        TickResult::Failure(reason)
    }
    pub fn running() -> TickResult {
        TickResult::Running
    }
}

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    CompileError(TreeError),
    UnImplementedAction(String),
    BlackBoardError(String),
    IOError(String),
    Unexpected(String),
    WrongArgument(String),
}

impl RuntimeError {
    pub fn uex(s: String) -> Self {
        Self::Unexpected(s)
    }
    pub fn bb(s: String) -> Self {
        Self::BlackBoardError(s)
    }
}

impl From<TreeError> for RuntimeError {
    fn from(value: TreeError) -> Self {
        RuntimeError::CompileError(value)
    }
}
