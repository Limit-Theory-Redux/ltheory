/// A process exit code: `libc::EXIT_SUCCESS` (0) or a failure.
pub type ExitCode = i32;

/// A request to end the run, and the process exit code it ends with.
///
/// The first non-zero code is kept, so a later request can't hide a failure.
#[derive(Debug, Default)]
pub struct ExitRequest {
    code: Option<ExitCode>,
}

impl ExitRequest {
    /// Asks to end the run with `code`; ignored when a failure code was already requested.
    pub fn request(&mut self, code: ExitCode) {
        if self
            .code
            .is_none_or(|current| current == libc::EXIT_SUCCESS)
        {
            self.code = Some(code);
        }
    }

    /// Whether any exit was requested.
    pub fn is_requested(&self) -> bool {
        self.code.is_some()
    }

    /// The requested code, or `EXIT_SUCCESS` when nothing was requested.
    pub fn code(&self) -> ExitCode {
        self.code.unwrap_or(libc::EXIT_SUCCESS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_request_is_success() {
        let request = ExitRequest::default();
        assert!(!request.is_requested());
        assert_eq!(request.code(), libc::EXIT_SUCCESS);
    }

    #[test]
    fn requested_code_is_returned() {
        let mut request = ExitRequest::default();
        request.request(3);
        assert!(request.is_requested());
        assert_eq!(request.code(), 3);
    }

    #[test]
    fn failure_replaces_success() {
        let mut request = ExitRequest::default();
        request.request(libc::EXIT_SUCCESS);
        request.request(2);
        assert_eq!(request.code(), 2);
    }

    #[test]
    fn first_failure_is_kept() {
        let mut request = ExitRequest::default();
        request.request(2);
        request.request(libc::EXIT_SUCCESS);
        request.request(5);
        assert_eq!(request.code(), 2);
    }
}
