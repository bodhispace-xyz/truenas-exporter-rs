# Releases

Merge application changes through the normal review process into `main`. Release
Please opens a version/changelog PR targeting `main`. Review it and wait for all
seven CI jobs, then merge it to create the release.

Release Please directly calls the reusable publisher with the tag it just created.
This avoids relying on another workflow starting from a tag created with
`GITHUB_TOKEN`. The publisher accepts the existing
`truenas-exporter-rs-v<version>` format and legacy `v<version>` tags. Existing tags
do not need to be renamed or moved.

An ordinary `main` push publishes only the `main` and full `sha-<commit>` container
tags. A stable release publishes the release tag, `v<version>`, `<version>`, minor
and major aliases, and `latest`. Its OCI labels identify the exact source commit
and package version. Versioned and `latest` tags therefore require a release.

Before publication, `Cargo.toml`, `Cargo.lock`, the Release Please manifest and the
release tag must agree. Builds use the locked dependencies. The GitHub release
receives `truenas-exporter-linux-amd64.tar.gz` and its `.sha256` checksum file.

## Repair a missing publication

After this workflow is merged into `main`, a maintainer can dispatch **Release**
from `main` with an existing release tag as `release_tag`. The workflow keeps the
current validation tooling but checks out and builds the exact tagged source.
For the incomplete 0.5.9 release, use `truenas-exporter-rs-v0.5.9`.

This operation also publishes `latest` and the version aliases, so repair the most
recent stable release before a newer version is published. Do not move or recreate
existing tags. The repair attaches assets to the existing GitHub release.

After publication, check workflow results, download and verify the archive against
its checksum, and verify that the versioned and `latest` container tags have the
same digest and expected source revision. A green Release Please job alone only
confirms release creation, not completion of publication.
