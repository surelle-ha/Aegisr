use colored::*;
use serde_json::Value;
use std::error::Error;
use std::fmt;

#[allow(dead_code)]
#[derive(Debug)]
pub enum TermenuError {
    Exception {
        code: String,
        name: String,
        data: Option<Value>,
    },
    Io(std::io::Error),
}

#[allow(dead_code)]
impl TermenuError {
    // Framework Errors (<011)
    pub fn framework_unknown_error(data: Option<Value>) -> Self {
        TermenuError::Exception {
            code: "E011".into(),
            name: "FrameworkError".into(),
            data,
        }
    }
    pub fn framework_forbidden_error(data: Option<Value>) -> Self {
        TermenuError::Exception {
            code: "E011".into(),
            name: "FrameworkForbiddenError".into(),
            data,
        }
    }
    pub fn framework_resource_error(data: Option<Value>) -> Self {
        TermenuError::Exception {
            code: "E011".into(),
            name: "FrameworkForbiddenError".into(),
            data,
        }
    }

    /// Input Errors (<111)
    pub fn input_unknown_error(data: Option<Value>) -> Self {
        TermenuError::Exception {
            code: "E111".into(),
            name: "InputError".into(),
            data,
        }
    }
    pub fn input_missing_error(data: Option<Value>) -> Self {
        TermenuError::Exception {
            code: "E112".into(),
            name: "InputMissingError".into(),
            data,
        }
    }

    /// Connection Errors (<611)
    pub fn connection_unknown_error(data: Option<Value>) -> Self {
        TermenuError::Exception {
            code: "E611".into(),
            name: "ConnectionError".into(),
            data,
        }
    }
    pub fn connection_timeout_error(data: Option<Value>) -> Self {
        TermenuError::Exception {
            code: "E612".into(),
            name: "ConnectionTimeoutError".into(),
            data,
        }
    }

    /// Command Errors (<711)
    pub fn command_unknown_error(data: Option<Value>) -> Self {
        TermenuError::Exception {
            code: "E711".into(),
            name: "CommandError".into(),
            data,
        }
    }
    pub fn invalid_command_error(data: Option<Value>) -> Self {
        TermenuError::Exception {
            code: "E712".into(),
            name: "InvalidCommandError".into(),
            data,
        }
    }
    pub fn command_timeout_error(data: Option<Value>) -> Self {
        TermenuError::Exception {
            code: "E713".into(),
            name: "CommandTimeoutError".into(),
            data,
        }
    }
}

impl fmt::Display for TermenuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TermenuError::Exception { code, name, data } => {
                let code_colored: ColoredString = code.red().bold();
                let name_colored: ColoredString = name.yellow().bold();

                if let Some(d) = data {
                    let issue = d
                        .get("issue")
                        .and_then(|v| v.as_str())
                        .unwrap_or("<no issue field>");

                    write!(f, "[{}] {}: {}", code_colored, name_colored, issue.cyan())
                } else {
                    write!(f, "[{}] {}", code_colored, name_colored)
                }
            }
            TermenuError::Io(err) => write!(f, "{}", err.to_string().red()),
        }
    }
}

impl Error for TermenuError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            TermenuError::Io(e) => Some(e),
            _ => None,
        }
    }
}
