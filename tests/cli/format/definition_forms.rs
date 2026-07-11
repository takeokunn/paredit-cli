use super::assert_format_output;

#[test]
fn cli_keeps_short_defsystem_on_one_line() {
    assert_format_output(
        "format-defsystem-short",
        "system.asd",
        "(defsystem \"foo\"\n  :description \"short\"\n  :version \"0.1.0\"\n  :depends-on (:asdf))\n",
        "(defsystem \"foo\" :description \"short\" :version \"0.1.0\" :depends-on (:asdf))\n",
    );
}

#[test]
fn cli_breaks_long_defsystem_keeping_option_pairs_together() {
    assert_format_output(
        "format-defsystem-long",
        "system.asd",
        "(defsystem \"my-really-quite-long-system-name\" :description \"a considerably longer description string here\" :version \"0.1.0\" :depends-on (:alexandria :bordeaux-threads))\n",
        "(defsystem \"my-really-quite-long-system-name\"\n  :description \"a considerably longer description string here\"\n  :version \"0.1.0\"\n  :depends-on (:alexandria :bordeaux-threads))\n",
    );
}
