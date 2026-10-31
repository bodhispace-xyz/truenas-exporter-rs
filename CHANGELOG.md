# Changelog

## [0.5.9](https://github.com/bodhispace-xyz/truenas-exporter-rs/compare/truenas-exporter-rs-v0.5.8...truenas-exporter-rs-v0.5.9) (2026-08-24)


### Features

* add boot pool, NFS/iSCSI client, and ZFS ARC metrics ([ec1a096](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/ec1a0969aacb3892918bc7af232a82adfd6ef3cb))
* add boot pool, NFS/iSCSI client, and ZFS ARC metrics (v0.5.6) ([86cf080](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/86cf080fbf8a9242e6e8aea0716c927fd279a39e))
* Add metric for last run smart test and hdd lifetime run ([b2cfd6e](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/b2cfd6eb516cd1742bbbf4c578cae7df63b17cfa))
* Add metric for last run smart test and hdd lifetime run ([975aae8](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/975aae8d10db2b4e33f2fff43f9fdf619d81b510))


### Bug Fixes

* address code review issues in boot pool/NFS/iSCSI collectors ([3c0b45c](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/3c0b45cbd26606087bfc49ab662a59f88dbb3197))
* address Copilot review comments ([228dea8](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/228dea89ece68640b63d9bb99f166208c073c2bd))
* **ci:** set target-branch to main for release-please ([d647374](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/d64737451096e0fc58789b6c6463ded1a62750a6))
* **ci:** set target-branch to main for release-please ([3b2fe3e](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/3b2fe3e9697f546a433e53683b6d779cad939bc1))
* clear stale state labels in snapshot and cloud sync metrics ([6217b9f](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/6217b9f143880cb9b35530f3c99f710c2988b0db))
* clear stale state labels in snapshot and cloud sync metrics ([bb5cb6b](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/bb5cb6b75990fc5affbc99cb46e5dcc7fe808656))
* Docker release tag invalid reference on tag push ([e24e7dd](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/e24e7ddb60c677b52c88026b17bd0f1d3aa80c76))
* Docker release tag invalid reference on tag push ([25a1752](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/25a175202805d56c63ffa91f5bf87088bd3b6dcf))
* remove state=time label pollution from memory metrics ([f0f3d1f](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/f0f3d1f538b14bb541fa027e375de113b2e714da))
* skip 'time' legend field in cpu and cputemp reporting arms ([8b6e04f](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/8b6e04fdd398de20c28efda1fb5682c32adeea3b))
* skip 'time' legend field in memory reporting to prevent state="time" label pollution ([c022ddb](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/c022ddb38e1ea399d55c8b52688195b7d65916e8))
* use .as_str() in with_label_values to satisfy &[&str] type ([d92b2da](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/d92b2da3a3dd392a45b84dbddda6eddb41c690c1))
* use `upgrade_available` for apps update available metric ([0b090bc](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/0b090bc786400973f6d47cc6075da67f25f10285))
* use static sha- prefix in Docker metadata tag to avoid empty branch name on tag push ([37b9ea3](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/37b9ea332ef1f69e3d986f762e313ee165cae009))


### Performance Improvements

* **ci:** speed up CI with pre-built cargo-audit and parallel release build ([27de569](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/27de5694772c3166d0e8e62d87f0c7f2302aff6e))
* **ci:** speed up CI with pre-built cargo-audit binary and parallel release build ([b663f69](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/b663f69649f2e103a4c412ffda91a16d3b098893))
* optimise memory usage with pre-sized collections ([9656e0f](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/9656e0f04396b3e369be0ed53855f0046de77ec3))
* optimize memory usage with pre-sized collections ([9843d8b](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/9843d8b1412e8bebe5148a923ed5960b0909350e))

## [0.5.8](https://github.com/bodhispace-xyz/truenas-exporter-rs/compare/truenas-exporter-rs-v0.5.7...truenas-exporter-rs-v0.5.8) (2026-08-22)


### Features

* add boot pool, NFS/iSCSI client, and ZFS ARC metrics ([ec1a096](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/ec1a0969aacb3892918bc7af232a82adfd6ef3cb))
* add boot pool, NFS/iSCSI client, and ZFS ARC metrics (v0.5.6) ([86cf080](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/86cf080fbf8a9242e6e8aea0716c927fd279a39e))
* Add metric for last run smart test and hdd lifetime run ([b2cfd6e](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/b2cfd6eb516cd1742bbbf4c578cae7df63b17cfa))
* Add metric for last run smart test and hdd lifetime run ([975aae8](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/975aae8d10db2b4e33f2fff43f9fdf619d81b510))


### Bug Fixes

* address code review issues in boot pool/NFS/iSCSI collectors ([3c0b45c](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/3c0b45cbd26606087bfc49ab662a59f88dbb3197))
* address Copilot review comments ([228dea8](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/228dea89ece68640b63d9bb99f166208c073c2bd))
* clear stale state labels in snapshot and cloud sync metrics ([6217b9f](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/6217b9f143880cb9b35530f3c99f710c2988b0db))
* clear stale state labels in snapshot and cloud sync metrics ([bb5cb6b](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/bb5cb6b75990fc5affbc99cb46e5dcc7fe808656))
* Docker release tag invalid reference on tag push ([25a1752](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/25a175202805d56c63ffa91f5bf87088bd3b6dcf))
* remove state=time label pollution from memory metrics ([f0f3d1f](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/f0f3d1f538b14bb541fa027e375de113b2e714da))
* skip 'time' legend field in cpu and cputemp reporting arms ([8b6e04f](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/8b6e04fdd398de20c28efda1fb5682c32adeea3b))
* skip 'time' legend field in memory reporting to prevent state="time" label pollution ([c022ddb](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/c022ddb38e1ea399d55c8b52688195b7d65916e8))
* use .as_str() in with_label_values to satisfy &[&str] type ([d92b2da](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/d92b2da3a3dd392a45b84dbddda6eddb41c690c1))
* use `upgrade_available` for apps update available metric ([0b090bc](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/0b090bc786400973f6d47cc6075da67f25f10285))
* use static sha- prefix in Docker metadata tag to avoid empty branch name on tag push ([37b9ea3](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/37b9ea332ef1f69e3d986f762e313ee165cae009))


### Performance Improvements

* optimise memory usage with pre-sized collections ([9656e0f](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/9656e0f04396b3e369be0ed53855f0046de77ec3))
* optimize memory usage with pre-sized collections ([9843d8b](https://github.com/bodhispace-xyz/truenas-exporter-rs/commit/9843d8b1412e8bebe5148a923ed5960b0909350e))
