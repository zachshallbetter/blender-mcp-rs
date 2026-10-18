use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TrajectoryFeedbackEntry {
    /// Evaluation signal: "accept", "reject", "undo", or "correction"
    pub feedback: String,
    /// Free-text correction or guidance from the user
    #[serde(default)]
    pub correction_text: Option<String>,
    /// Step index in current agent trajectory session
    #[serde(default)]
    pub step_index: Option<usize>,
    /// User prompt / goal description
    #[serde(default)]
    pub user_prompt: Option<String>,
    /// Unix timestamp in seconds
    pub timestamp_sec: u64,
}

#[derive(Clone, Default)]
pub struct TrajectoryRecorder {
    entries: Arc<Mutex<Vec<TrajectoryFeedbackEntry>>>,
}

impl TrajectoryRecorder {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn record_feedback(
        &self,
        feedback: &str,
        correction_text: Option<String>,
        step_index: Option<usize>,
        user_prompt: Option<String>,
    ) -> Result<String, String> {
        let valid_signals = ["accept", "reject", "undo", "correction"];
        if !valid_signals.contains(&feedback) {
            return Err(format!(
                "Invalid feedback signal '{}'. Must be one of {:?}",
                feedback, valid_signals
            ));
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let entry = TrajectoryFeedbackEntry {
            feedback: feedback.to_string(),
            correction_text,
            step_index,
            user_prompt,
            timestamp_sec: now,
        };

        let mut lock = self.entries.lock().map_err(|e| e.to_string())?;
        lock.push(entry);

        Ok(format!(
            "Trajectory feedback '{}' recorded successfully (total entries: {})",
            feedback,
            lock.len()
        ))
    }

    pub fn entries(&self) -> Vec<TrajectoryFeedbackEntry> {
        self.entries.lock().map(|l| l.clone()).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_trajectory_feedback() {
        let recorder = TrajectoryRecorder::new();
        let res = recorder.record_feedback(
            "accept",
            None,
            Some(0),
            Some("Add barrel to scene".to_string()),
        );
        assert!(res.is_ok());
        assert_eq!(recorder.entries().len(), 1);

        let res2 = recorder.record_feedback(
            "correction",
            Some("Make it slightly taller".to_string()),
            Some(1),
            Some("Refine barrel height".to_string()),
        );
        assert!(res2.is_ok());
        assert_eq!(recorder.entries().len(), 2);

        let invalid = recorder.record_feedback("bad_signal", None, None, None);
        assert!(invalid.is_err());
    }
}
