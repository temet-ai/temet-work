# Releasing temet-work

## Version and artifacts

One workspace SemVer version, `0.1.0-alpha.1`, then the last number goes up for each
alpha. Commit the version and the lockfile together. An annotated `v<version>` tag on
`main` of `temet-ai/temet-work` runs `.github/workflows/release.yml`: it refuses a tag that
disagrees with Cargo, tests, builds each target, checks `temet-work --version`, and
publishes a GitHub release (a prerelease while the version has `-alpha`) with SHA-256
checksums. Never move a released tag or replace its assets; publish the next version.

Targets: macOS Apple Silicon and Intel, Linux arm64 and x64 (glibc, built on Ubuntu
22.04, so glibc 2.35 or later). Each archive holds `temet-work`, LICENSE and
THIRD-PARTY-NOTICES.md. Not built: Windows (WSL2 meanwhile), older glibc, other
architectures. macOS binaries are unsigned; Homebrew and mise are the supported routes.

## Distribution

The source is public at [`temet-ai/temet-work`](https://github.com/temet-ai/temet-work), and its
releases are the one source for package managers. It is developed in the open in that
repository: there is no private copy and no export step. Every commit is public, so the
hooks in `scripts/hooks` (the private word list, `scripts/scan-secrets`) run before each
one.

Homebrew: the tap `temet-ai/homebrew-temet-work` holds `Formula/temet-work.rb`, rendered from a
release's `checksums.txt` by `scripts/render-homebrew-formula`. Install docs always give
the full name, `brew install temet-ai/temet-work/temet-work`: a formula or cask elsewhere with
the same short name wins over a tap for `brew install temet-work`.

mise:

```toml
[tools]
"github:temet-ai/temet-work" = { version = "latest", prerelease = true, minimum_release_age = "0s" }
```

`prerelease = true` because alphas are prereleases; `minimum_release_age = "0s"` because
mise otherwise holds a new GitHub release for 24 hours.

## Publishing checklist

1. On `main`: bump the version, run `scripts/notices`, the tests, clippy with
   `-D warnings` and `cargo deny check`. Commit and push.
2. Create and push an annotated tag `v<version>` on that commit.
3. Wait for the build jobs and the publish job to succeed; four archives and `checksums.txt`.
4. With `HOMEBREW_TAP_TOKEN` set on `temet-ai/temet-work` (a fine-grained token that can write
   only to `temet-ai/homebrew-temet-work`) the publish job commits the formula. Without it:
   download `checksums.txt`, run `scripts/render-homebrew-formula <version> temet-ai/temet-work
   checksums.txt > Formula/temet-work.rb` in a clone of the tap, commit and push.
5. Upgrade through brew on the Mac and mise on Linux, from a shell with a clean `PATH`, and
   run it: `temet-work --version` shows the new version, LICENSE and the notices are in the prefix.

The release workflow needs only `contents: write` on its own repository;
`HOMEBREW_TAP_TOKEN` is the one other credential.
