# Releasing

One version number covers the whole release: the Rust crates, the WASM, the
wrapper and the npm package. It is `[workspace.package].version` in
`Cargo.toml`. Channels follow spec §26.4:

- `-alpha.N`: a prototype; the API may change.
- `-beta.N`: everything planned works and testing is under way.
- `1.0.0`: the API and the game are stable.

## Cutting a release

1. **Pick the version.** Set it in `Cargo.toml` and run
   `cargo update --workspace` so `Cargo.lock` follows.
2. **Write the changelog.** Add a `## [x.y.z] - YYYY-MM-DD` section to
   `CHANGELOG.md`; the release workflow uses it as the release notes. Flag
   anything that changes shot outcomes, and bump the physics profile id with
   it.
3. **Merge to `main`** through a pull request with green CI.
4. **Tag and push:** `git tag vX.Y.Z && git push origin vX.Y.Z`. The
   `Release` workflow:
   - checks that the tag matches the workspace version;
   - runs every test, the stress test included;
   - builds the bundle and runs the clean-consumer test;
   - publishes a GitHub release with the npm tarball, a zip of the demo and
     `SHA256SUMS`. Versions with a hyphen are marked as pre-releases.
5. **Publish to npm by hand.** This is deliberate: it needs a member of the
   npm organization `tuletta`, which enforces a second factor. Download the
   tarball from the release, check it against `SHA256SUMS`, then run:

   ```sh
   npm login
   grep tuletta-cue-X.Y.Z.tgz SHA256SUMS | shasum -a 256 -c
   npm publish tuletta-cue-X.Y.Z.tgz --access public
   ```

   npm asks for the one-time code from the authenticator, or takes it as
   `--otp=123456`. Pre-releases go out under a `beta` tag (`--tag beta`) so
   they never become `latest`: hosts install them with
   `npm install @tuletta/cue@beta`.
6. **Update the portfolio** to the new version, keeping the previous one
   ready for a rollback (spec §26.5).

## Rolling back

Every release keeps its tarball and its demo zip. To roll back, point the
host at the previous version, or deploy the previous demo zip. Nothing else
needs undoing: CUE has no backend and no data to migrate.
