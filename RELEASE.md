# Releasing civterm

## Prerequisites

- A clean working tree on the default branch.
- `cargo-release` installed: `cargo install cargo-release`
- (Optional) `HOMEBREW_TAP_TOKEN` / `AUR_SSH_KEY` secrets set in the GitHub repo for automated tap/AUR updates.

## Release a new version

1. Make sure the tree is clean and `make build` passes.
2. Run:

   ```bash
   cargo release 0.9.4 --no-publish --execute
   ```

   This bumps the version in `Cargo.toml`/`Cargo.lock`, commits, tags `v0.9.4`,
   pushes, and triggers the `release.yml` workflow (which builds all binaries,
   packages, attestations, and publishes the GitHub release).

3. Update the Homebrew tap:

   - Open the GitHub Actions run for the release and download the
     `homebrew-formula` artifact.
   - Copy the contents of the downloaded `Formula/civterm.rb` and paste them
     into `Formula/civterm.rb` in the `homebrew-civterm` tap repo.
   - Commit and push:

     ```bash
     cd /path/to/homebrew-civterm
     git add Formula/civterm.rb
     git commit -m "Update civterm to v0.9.4"
     git push
     ```

## Notes

- The tag must match the `Cargo.toml` version (the workflow enforces this).
- `publish = false` is set, so nothing is published to crates.io.
- For the first release of a new minor/major, adjust the version argument
  (e.g. `cargo release 1.0.0 --no-publish --execute`).