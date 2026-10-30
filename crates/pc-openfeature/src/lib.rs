use open_feature::EvaluationContext;
use pc_types::StageReceipt;
pub fn evaluate(input: &str) -> StageReceipt {
    if input.trim().is_empty() {
        return StageReceipt::deny("openfeature", "empty context");
    };
    let c = EvaluationContext::default().with_targeting_key("patch-cage-review");
    StageReceipt::pass(
        "openfeature",
        format!("evaluated {}", std::any::type_name_of_val(&c)),
        1.0,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn evaluates_context() {
        assert_eq!(evaluate("release").status, "passed")
    }
}
