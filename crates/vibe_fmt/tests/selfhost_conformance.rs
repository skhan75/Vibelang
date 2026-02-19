use std::fs;
use std::path::PathBuf;

use vibe_fmt::format_source;

#[test]
fn host_formatter_matches_selfhost_fixture_outputs() {
    for fixture in fixture_names() {
        let input = fs::read_to_string(fixtures_root().join(format!("{fixture}.input")))
            .expect("read input");
        let selfhost = fs::read_to_string(fixtures_root().join(format!("{fixture}.selfhost.out")))
            .expect("read selfhost output");
        let host = format_source(&input);
        assert_eq!(
            host, selfhost,
            "fixture `{fixture}` diverged between host formatter and selfhost prototype output"
        );
    }
}

#[test]
fn host_formatter_repeat_runs_are_deterministic() {
    for fixture in fixture_names() {
        let input = fs::read_to_string(fixtures_root().join(format!("{fixture}.input")))
            .expect("read input");
        let first = format_source(&input);
        let second = format_source(&first);
        let third = format_source(&second);
        assert_eq!(first, second, "first/second outputs differ for `{fixture}`");
        assert_eq!(second, third, "second/third outputs differ for `{fixture}`");
    }
}

fn fixture_names() -> Vec<&'static str> {
    vec!["basic", "nested"]
}

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("selfhost")
        .join("fixtures")
}
