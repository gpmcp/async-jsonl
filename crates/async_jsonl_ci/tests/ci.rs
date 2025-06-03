use async_jsonl_ci::workflow;

#[test]
fn generate() {
    workflow::generate_ci_workflow();
}

#[test]
fn test_release_drafter() {
    workflow::generate_release_drafter_workflow();
}

#[test]
fn test_release_plz() {
    workflow::generate_release_plz_workflow();
}
