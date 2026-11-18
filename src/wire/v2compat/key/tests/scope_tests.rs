use super::scope_api as scope;
#[test]
fn scopes_match_v2_prefix_semantics() {
    for (loc, key, expected) in [
        ("*", "anything", true),
        ("", "anything", false),
        ("a", "a", true),
        ("a", "ab", false),
        ("a*", "ab", true),
        ("a*", "a", true),
        ("a/*", "a/x", true),
        ("a/*", "ab/x", false),
        ("a/*", "a", false),
    ] {
        assert_eq!(scope::matches(loc, key), expected, "{loc} vs {key}");
    }
}
#[test]
fn scope_validation_and_subsets_are_exact() {
    for (loc, expected) in [
        ("", true),
        ("*", true),
        ("a", true),
        ("a*", true),
        ("a/*", true),
        ("a.b", true),
        (".a", false),
        ("a/**", false),
        ("a//b", false),
        ("a *", false),
        ("*/a", false),
    ] {
        assert_eq!(scope::valid_scope(loc), expected, "{loc}");
    }
    for (child, parent, expected) in [
        ("", "", true),
        ("", "a", true),
        ("a", "*", true),
        ("a*", "a", false),
        ("a/b", "a/*", true),
        ("a2", "a/*", false),
        ("a*", "a*", true),
        ("ab*", "a*", true),
        ("b", "a*", false),
    ] {
        assert_eq!(scope::subset(child, parent), expected, "{child}, {parent}");
    }
}
