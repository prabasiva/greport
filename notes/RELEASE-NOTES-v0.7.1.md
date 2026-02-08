# Release v0.7.1 -- CI Build Fix

## Summary

This patch release fixes the musl cross-compilation failure in the release CI pipeline. The `x86_64-unknown-linux-musl` target was failing because the GNU-targeted OpenSSL headers are incompatible with the musl toolchain. The fix uses vendored OpenSSL (compiled from source) for the musl target only.

## Bug Fixes

- Fixed release CI build failure for the `x86_64-unknown-linux-musl` target by using vendored OpenSSL
- Added `vendored-openssl` feature flag to `greport-cli` and `greport-api` crates, which enables `openssl/vendored` to compile OpenSSL from source during the musl build
- Release workflow passes `--features vendored-openssl` only for the musl target, keeping all other targets unchanged
- Moved release notes from `docs/` to `notes/` directory and updated the release workflow path accordingly

## Files Changed

### Modified (4)
- `Cargo.toml` -- Added `openssl` workspace dependency
- `crates/greport-api/Cargo.toml` -- Added `vendored-openssl` feature and optional `openssl` dependency
- `crates/greport-cli/Cargo.toml` -- Added `vendored-openssl` feature and optional `openssl` dependency
- `.github/workflows/release.yml` -- Use vendored OpenSSL for musl target, read release notes from `notes/`

### Added (2)
- `notes/RELEASE-NOTES-v0.6.0.md` -- Moved from `docs/`
- `notes/RELEASE-NOTES-v0.7.1.md` -- This release

## Upgrade Notes

- No application code changes
- No breaking changes
- The `vendored-openssl` feature is opt-in and only used in CI for musl builds
