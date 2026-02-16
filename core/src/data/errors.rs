#[derive(Clone, Debug)]
pub struct DataError {
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, Default)]
pub struct ErrorReport {
    pub errors: Vec<DataError>,
}

impl ErrorReport {
    pub fn push(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.errors.push(DataError {
            path: path.into(),
            message: message.into(),
        });
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn join_messages(&self) -> String {
        self.errors
            .iter()
            .map(|e| format!("{}: {}", e.path, e.message))
            .collect::<Vec<_>>()
            .join(" | ")
    }
}
