use pc_types::StageReceipt;
use ruvector_core::{VectorDB, VectorEntry};
pub fn evaluate(input: &str) -> StageReceipt {
    if input.trim().is_empty() {
        return StageReceipt::deny("ruvector", "empty patch");
    };
    let db = match VectorDB::with_dimensions(4) {
        Ok(x) => x,
        Err(e) => return StageReceipt::deny("ruvector", e.to_string()),
    };
    let v = vec![
        input.len() as f32,
        input.lines().count() as f32,
        input.matches('+').count() as f32,
        input.matches('-').count() as f32,
    ];
    match db.insert(VectorEntry {
        id: None,
        vector: v,
        metadata: None,
    }) {
        Ok(id) => StageReceipt::pass("ruvector", format!("inserted precedent {id}"), 1.0),
        Err(e) => StageReceipt::deny("ruvector", e.to_string()),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inserts_vector() {
        assert_eq!(evaluate("diff --git a/a b/a").status, "passed")
    }
}
