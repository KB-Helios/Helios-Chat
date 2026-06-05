use super::types::EieLogLine;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_LOG_LINES: usize = 500;

#[derive(Clone, Default)]
pub struct EieLogBuffer {
    lines: Arc<Mutex<VecDeque<EieLogLine>>>,
}

impl EieLogBuffer {
    pub fn push(&self, stream: impl Into<String>, line: impl Into<String>) -> EieLogLine {
        let entry = EieLogLine {
            stream: stream.into(),
            line: line.into(),
            timestamp: timestamp(),
        };

        let mut lines = self.lines.lock().expect("EIE log buffer poisoned");
        lines.push_back(entry.clone());
        while lines.len() > MAX_LOG_LINES {
            lines.pop_front();
        }

        entry
    }

    pub fn list(&self) -> Vec<EieLogLine> {
        self.lines
            .lock()
            .expect("EIE log buffer poisoned")
            .iter()
            .cloned()
            .collect()
    }

    pub fn clear(&self) {
        self.lines
            .lock()
            .expect("EIE log buffer poisoned")
            .clear();
    }
}

pub fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
