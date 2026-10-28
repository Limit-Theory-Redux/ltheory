/// A request to end the run, and the process exit code it ends with.
///
/// The first non-zero code is kept, so a later request can't hide a failure.
#[derive(Debug, Default)]
pub struct ExitRequest {
    code: Option<i32>,
}

impl ExitRequest {
    pub fn request(&mut self, code: i32) {
        if self
            .code
            .is_none_or(|current| current == libc::EXIT_SUCCESS)
        {
            self.code = Some(code);
        }
    }

    pub fn is_requested(&self) -> bool {
        self.code.is_some()
    }

    /// The requested code, or `EXIT_SUCCESS` when nothing was requested.
    pub fn code(&self) -> i32 {
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
