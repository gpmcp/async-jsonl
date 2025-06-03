use gh_workflow_tailcall::*;

/// Create a Release-plz workflow for automated releases
pub fn create_release_plz_workflow() -> Workflow {
    let mut release_plz_workflow = Workflow::default()
        .name("Release-plz")
        .on(Event {
            push: Some(Push {
                branches: vec!["main".to_string()],
                ..Push::default()
            }),
            ..Event::default()
        })
        .permissions(
            Permissions::default()
                .pull_requests(Level::Write)
                .contents(Level::Write),
        );

    // Add the release-plz release job
    release_plz_workflow =
        release_plz_workflow.add_job("release-plz-release", create_release_plz_release_job());

    // Add the release-plz PR job
    release_plz_workflow =
        release_plz_workflow.add_job("release-plz-pr", create_release_plz_pr_job());

    release_plz_workflow
}

/// Create a job to release unpublished packages
pub fn create_release_plz_release_job() -> Job {
    Job::new("release-plz-release")
        .name("Release-plz release")
        .runs_on("ubuntu-latest")
        .permissions(Permissions::default().contents(Level::Write))
        .add_step(
            Step::uses("actions", "checkout", "v4")
                .name("Checkout repository")
                .with(("fetch-depth", "0")),
        )
        .add_step(Step::uses("dtolnay", "rust-toolchain", "stable").name("Install Rust toolchain"))
        .add_step(
            Step::uses("release-plz", "action", "v0.5")
                .name("Run release-plz")
                .with(("command", "release"))
                .env(("GITHUB_TOKEN", "${{ secrets.GH_TOKEN }}"))
                .env((
                    "CARGO_REGISTRY_TOKEN",
                    "${{ secrets.CARGO_REGISTRY_TOKEN }}",
                )),
        )
}

/// Create a job to create/update release PR
pub fn create_release_plz_pr_job() -> Job {
    Job::new("release-plz-pr")
        .name("Release-plz PR")
        .runs_on("ubuntu-latest")
        .permissions(
            Permissions::default()
                .contents(Level::Write)
                .pull_requests(Level::Write),
        )
        .concurrency(Concurrency {
            group: "release-plz-${{ github.ref }}".to_string(),
            cancel_in_progress: Some(false),
            limit: None,
        })
        .add_step(
            Step::uses("actions", "checkout", "v4")
                .name("Checkout repository")
                .with(("fetch-depth", "0")),
        )
        .add_step(Step::uses("dtolnay", "rust-toolchain", "stable").name("Install Rust toolchain"))
        .add_step(
            Step::uses("release-plz", "action", "v0.5")
                .name("Run release-plz")
                .with(("command", "release-pr"))
                .env(("GITHUB_TOKEN", "${{ secrets.GH_TOKEN }}"))
                .env((
                    "CARGO_REGISTRY_TOKEN",
                    "${{ secrets.CARGO_REGISTRY_TOKEN }}",
                )),
        )
}
