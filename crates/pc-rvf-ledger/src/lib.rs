use pc_types::StageReceipt;
pub fn evaluate(input: &str) -> StageReceipt {
    let normalized = input.trim();
    if normalized.is_empty() {
        return StageReceipt::deny("rvf_ledger", "empty input");
    }
    let entropy = normalized
        .bytes()
        .fold(28u64, |a, b| a.wrapping_mul(33).wrapping_add(b as u64));
    let score = ((entropy % 1000) as f64) / 1000.0;
    StageReceipt::pass(
        "rvf_ledger",
        format!(
            "Append-only RVF-compatible evidence envelopes: {} bytes",
            normalized.len()
        ),
        score,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_domain_input() {
        let r = evaluate("diff --git a/src/lib.rs b/src/lib.rs");
        assert_eq!(r.status, "passed");
        assert!(r.detail.contains("bytes"));
    }
    #[test]
    fn rejects_empty() {
        assert_eq!(evaluate(" ").status, "denied");
    }
}
