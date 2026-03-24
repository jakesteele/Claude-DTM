use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SessionStatus {
    Running,
    Waiting,
    Done,
    Paused,
}

impl SessionStatus {
    pub fn label(&self) -> &'static str {
        match self {
            SessionStatus::Running => "RUNNING",
            SessionStatus::Waiting => "WAITING",
            SessionStatus::Done => "DONE",
            SessionStatus::Paused => "PAUSED",
        }
    }
}

/// Detect session status from terminal screen content and timing.
pub fn detect_status(
    parser: &vt100::Parser,
    last_output_time: Instant,
    process_alive: bool,
) -> SessionStatus {
    if !process_alive {
        return SessionStatus::Done;
    }

    let idle_secs = last_output_time.elapsed().as_secs_f64();
    let screen = parser.screen();

    // If idle for 2+ seconds, check if it looks like a prompt
    if idle_secs >= 2.0 {
        let (_rows, cols) = screen.size();
        // rows() yields one String per visible row
        let row_texts: Vec<String> = screen.rows(0, cols).collect();

        // Check the last few non-empty rows for prompt indicators
        for text in row_texts.iter().rev().take(5) {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                continue;
            }
            if trimmed.ends_with('>')
                || trimmed.ends_with('❯')
                || trimmed.ends_with('$')
                || trimmed.ends_with('%')
                || trimmed.contains("❯")
                || trimmed.contains("> ")
            {
                return SessionStatus::Waiting;
            }
            break;
        }
    }

    SessionStatus::Running
}
