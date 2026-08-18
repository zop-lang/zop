use std::collections::HashSet;

use naga::valid::{Capabilities, ValidationFlags, Validator};
use serde::Deserialize;
use zop::{backend::javascript_text, frontend::analyze};

const PROFILE_SOURCE: &str = include_str!("../conformance/web/profile.toml");
const WGSL_FIXTURE: &str = include_str!("../conformance/web/fixtures/reference-affine.wgsl");
const KNOWN_TESTS: &[&str] = &[
    "generated_javascript_avoids_runtime_source_evaluation",
    "i32_arithmetic_retains_its_machine_width",
    "printer_preserves_tokens_and_uses_compact_float_literals",
    "scalar_hir_emits_a_deterministic_es_module",
    "web_profile_is_well_formed",
    "wgsl_reference_fixture_is_valid",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WebProfile {
    schema: u32,
    name: String,
    target: String,
    status: String,
    suites: Vec<Suite>,
    requirements: Vec<Requirement>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Suite {
    id: String,
    repository: String,
    revision: String,
    mode: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum RequirementStatus {
    Supported,
    Partial,
    Blocked,
    OutOfScope,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Requirement {
    id: String,
    area: String,
    status: RequirementStatus,
    spec: String,
    suites: Vec<String>,
    tests: Vec<String>,
}

#[test]
fn web_profile_is_well_formed() {
    let profile: WebProfile = toml::from_str(PROFILE_SOURCE).expect("web profile should parse");

    assert_eq!(profile.schema, 1);
    assert_eq!(profile.name, "web-2026q3");
    assert_eq!(profile.target, "js-browser");
    assert_eq!(profile.status, "experimental");

    let mut suite_ids = HashSet::new();
    for suite in &profile.suites {
        assert!(suite_ids.insert(suite.id.as_str()), "duplicate suite: {}", suite.id);
        assert!(suite.repository.starts_with("https://github.com/"), "{} repository", suite.id);
        assert!(is_git_revision(&suite.revision), "{} revision", suite.id);
        assert_eq!(suite.mode, "selected", "{} mode", suite.id);
    }

    let known_tests = KNOWN_TESTS.iter().copied().collect::<HashSet<_>>();
    let mut requirement_ids = HashSet::new();
    let mut referenced_tests = HashSet::new();
    for requirement in &profile.requirements {
        assert!(
            requirement_ids.insert(requirement.id.as_str()),
            "duplicate requirement: {}",
            requirement.id
        );
        assert!(!requirement.area.is_empty(), "{} area", requirement.id);
        assert!(requirement.spec.starts_with("https://"), "{} spec", requirement.id);
        assert!(!requirement.suites.is_empty(), "{} suites", requirement.id);
        assert!(
            requirement.suites.iter().all(|suite| suite_ids.contains(suite.as_str())),
            "{} references an unknown suite",
            requirement.id
        );
        match requirement.status {
            RequirementStatus::Supported => {
                assert!(!requirement.tests.is_empty(), "{} has no tests", requirement.id);
            }
            RequirementStatus::Blocked => {
                assert!(
                    requirement.tests.is_empty(),
                    "{} is blocked but claims tests",
                    requirement.id
                );
            }
            RequirementStatus::Partial | RequirementStatus::OutOfScope => {}
        }
        for test in &requirement.tests {
            assert!(known_tests.contains(test.as_str()), "unknown test: {test}");
            referenced_tests.insert(test.as_str());
        }
    }

    assert_eq!(referenced_tests, known_tests, "profile test coverage");
}

#[test]
fn generated_javascript_avoids_runtime_source_evaluation() {
    let source = "fn affine value: f64, scale: f64 -> f64\n    value * scale\n";
    let hir = analyze(source).expect("source should type-check");
    let javascript = javascript_text(&hir).expect("JavaScript should lower");

    for forbidden in ["eval(", "new Function", "Function("] {
        assert!(!javascript.contains(forbidden), "generated JavaScript contains {forbidden}");
    }
}

#[test]
fn wgsl_reference_fixture_is_valid() {
    let module = naga::front::wgsl::parse_str(WGSL_FIXTURE).expect("WGSL should parse");
    Validator::new(ValidationFlags::all(), Capabilities::empty())
        .validate(&module)
        .expect("WGSL should validate");
}

fn is_git_revision(revision: &str) -> bool {
    revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit())
}
