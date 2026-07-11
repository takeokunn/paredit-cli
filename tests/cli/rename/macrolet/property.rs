use super::*;

proptest! {
    #![proptest_config(cli_proptest_config(24))]

    #[test]
    fn pbt_cli_rename_macrolet_output_remains_parseable_and_preserves_inner_body_refs(
        from in "[a-z][a-z0-9-]{0,8}",
        to in "[a-z][a-z0-9-]{0,8}",
    ) {
        assert_cli_rename_macrolet_property(from, to)?;
    }
}
