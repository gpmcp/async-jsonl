use gh_workflow_tailcall::StandardWorkflow;

#[test]
fn generate() {
    StandardWorkflow::default()
        .auto_release(true)
        .auto_fix(true)
        .generate()
        .unwrap();
}
