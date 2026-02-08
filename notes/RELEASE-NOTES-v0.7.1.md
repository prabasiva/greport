# Release v0.7.1 -- CI Build Fix

## Summary

This patch release fixes the musl cross-compilation failure in the release CI pipeline. The `x86_64-unknown-linux-musl` target was failing because OpenSSL development libraries were not installed in the build environment.

## Bug Fixes

- Fixed release CI build failure for the `x86_64-unknown-linux-musl` target by installing `musl-dev`, `libssl-dev`, and `pkg-config` alongside `musl-tools`
- Set `PKG_CONFIG_ALLOW_CROSS=1` environment variable to enable pkg-config to locate OpenSSL libraries during musl cross-compilation

## Files Changed

### Modified (1)
- `.github/workflows/release.yml` -- Added OpenSSL dev dependencies and cross-compilation environment variable for musl builds

## Upgrade Notes

- No application code changes
- No breaking changes
- This fix only affects the release CI pipeline
