use crate::jobs;
use generate::Generate;
use gh_workflow_tailcall::*;

/// Generate the main CI workflow
pub fn generate_ci_workflow() {
    let workflow = StandardWorkflow::default()
        .auto_fix(true)
        .to_ci_workflow()
        .concurrency(Concurrency {
            group: "${{ github.workflow }}-${{ github.ref }}".to_string(),
            cancel_in_progress: None,
            limit: None,
        });

    // Generate the workflow with build job only
    workflow.generate().unwrap();
}

/// Generate release-plz workflow for automated releases
pub fn generate_release_plz_workflow() {
    let release_plz_workflow = jobs::create_release_plz_workflow();

    Generate::new(release_plz_workflow)
        .name("release-plz.yml")
        .generate()
        .unwrap();
}

/// Generate release drafter workflow (deprecated - keeping for compatibility)
pub fn generate_release_drafter_workflow() {
    let release_drafter_workflow = jobs::create_release_drafter_workflow();

    Generate::new(release_drafter_workflow)
        .name("release-drafter.yml")
        .generate()
        .unwrap();
}
