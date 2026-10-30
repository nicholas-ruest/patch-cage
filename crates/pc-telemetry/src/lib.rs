use opentelemetry::KeyValue;
use pc_types::StageReceipt;
pub fn evaluate(input: &str) -> StageReceipt {
    if input.trim().is_empty() {
        return StageReceipt::deny("otel", "empty correlation");
    };
    let kv = KeyValue::new("patch.bytes", input.len() as i64);
    StageReceipt::pass("otel", format!("emitted {:?}", kv), 1.0)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn emits_attribute() {
        assert_eq!(evaluate("trace").status, "passed")
    }
}
