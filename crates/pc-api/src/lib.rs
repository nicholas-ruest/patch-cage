pub const API_VERSION: &str = "v1";
#[cfg(test)]
mod tests {
    #[test]
    fn version() {
        assert_eq!(super::API_VERSION, "v1")
    }
}
