pub fn validate_schema(schema: &str) -> bool {
    schema
        .chars()
        .all(|c| c == '_' || c.is_ascii_alphanumeric())
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[test_case("test" => true)]
    #[test_case("test;" => false)]
    #[test_case("test4" => true)]
    #[test_case("te_st4" => true)]
    #[test_case("te st4" => false)]
    #[test_case("test4`" => false)]
    #[test_case("test4\n" => false)]
    #[test_case("test4$1" => false)]
    fn validate_schema_tests(schema: &str) -> bool {
        validate_schema(schema)
    }
}
