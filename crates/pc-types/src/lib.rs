use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StageReceipt {
    pub stage: String,
    pub status: String,
    pub detail: String,
    pub score: f64,
}
impl StageReceipt {
    pub fn pass(stage: &str, detail: impl Into<String>, score: f64) -> Self {
        Self {
            stage: stage.into(),
            status: "passed".into(),
            detail: detail.into(),
            score,
        }
    }
    pub fn deny(stage: &str, detail: impl Into<String>) -> Self {
        Self {
            stage: stage.into(),
            status: "denied".into(),
            detail: detail.into(),
            score: 0.0,
        }
    }
}
pub fn evaluate(input: &str) -> StageReceipt {
    if input.trim().is_empty() {
        StageReceipt::deny("types", "empty input")
    } else {
        StageReceipt::pass("types", "typed contracts", 1.0)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_empty() {
        assert_eq!(evaluate(" ").status, "denied")
    }
}
