# Release Process

This document describes how to release a new version of typeglass-cli.

## Prerequisites

- Commit access to the repository
- `CARGO_REGISTRY_TOKEN` set in GitHub secrets
- Clean working directory on main branch
- All tests passing

## Release Checklist

### 1. Prepare Release

```bash
# Ensure you're on main branch with latest changes
git checkout main
git pull origin main

# Run full test suite
cargo test --all-features

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check coverage (optional but recommended)
cargo tarpaulin
```

### 2. Run Release Script

```bash
# Run the automated release script
./scripts/release.sh X.Y.Z
```

The script will:
- Validate version format
- Check for clean working directory
- Update `Cargo.toml` and `Cargo.lock`
- Run tests and clippy
- Build release binary
- Generate changelog entry
- Create git commit and tag

### 3. Review Changes

```bash
# Review the changelog
cat CHANGELOG.md

# Review the commit
git show HEAD

# Review the tag
git show vX.Y.Z
```

### 4. Push Release

```bash
# Push the version bump commit
git push origin main

# Push the tag (this triggers GitHub Actions release workflow)
git push origin vX.Y.Z
```

### 5. Monitor Release

1. Go to GitHub Actions tab
2. Watch the "Release" workflow
3. Verify builds complete for all platforms
4. Check that binaries are attached to GitHub release
5. Verify crates.io publication

### 6. Announce Release (Optional)

- Update README if needed
- Post announcement
- Share release notes

## Version Numbering

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version: Breaking changes
- **MINOR** version: New features (backward compatible)
- **PATCH** version: Bug fixes (backward compatible)

## Rollback

If something goes wrong:

```bash
# Delete local tag
git tag -d vX.Y.Z

# Delete remote tag
git push origin :refs/tags/vX.Y.Z

# Reset commit
git reset --hard HEAD~1
git push origin main --force  # Use with caution!
```

## Troubleshooting

### Release workflow fails

- Check GitHub Actions logs
- Verify secrets are set correctly
- Ensure all required files are committed

### crates.io publication fails

- Check `CARGO_REGISTRY_TOKEN` secret
- Verify Cargo.toml metadata is correct
- Ensure version doesn't already exist

### Build fails on specific platform

- Check platform-specific dependencies
- Test locally with cross-compilation
- Review platform-specific code paths
