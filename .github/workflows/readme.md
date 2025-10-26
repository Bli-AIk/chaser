1. **CI Workflow** (`.github/workflows/ci.yml`)

    * Triggered on pushes to the `main`/`develop` branches and on PR creation
    * Tests on multiple Rust versions (stable, beta, nightly)
    * Includes code formatting checks, Clippy linting, build, and test
    * Builds binaries for Linux, Windows, and macOS
    * Includes security audit checks

2. **Release Workflow** (`.github/workflows/release.yml`)

    * Triggered when pushing a tag (v*)
    * Automatically creates a GitHub Release
    * Builds and uploads binaries for multiple platforms
    * Optionally publishes to crates.io (requires CRATES_TOKEN configuration)

3. **Dependency Update Workflow** (`.github/workflows/update-deps.yml`)

    * Runs automatically every Monday
    * Updates dependency versions in `Cargo.lock`
    * Automatically creates a PR if updates are available

4. **Code Coverage Workflow** (`.github/workflows/coverage.yml`)

    * Generates code coverage reports
    * Uploads to Codecov (optional)

5. **Dependabot Configuration** (`.github/dependabot.yml`)

    * Automatically monitors Rust dependencies and GitHub Actions updates
    * Creates update PRs weekly
