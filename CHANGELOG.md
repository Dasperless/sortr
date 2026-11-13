# 1.0.0 (2026-11-13)


### Bug Fixes

* let set-version.sh refresh Cargo.lock without an offline cache ([c547701](https://github.com/Dasperless/sortr/commit/c5477019de4393bae9cb18b7a96c72d204ecad49)), closes [#31](https://github.com/Dasperless/sortr/issues/31)


### Features

* add --dry-run and report module ([a17725e](https://github.com/Dasperless/sortr/commit/a17725e8e6650ba3fd8ca2073bae5718f9ae520a))
* add 'sortr undo' to revert the latest run ([1a6b3bb](https://github.com/Dasperless/sortr/commit/1a6b3bbd5baedf0e4e9d274822d6bef3ae5d39ab))
* add clap CLI wiring scan, plan and move ([5f055ad](https://github.com/Dasperless/sortr/commit/5f055ad97723da4ce1dc73399ccf48c1aedc9adc))
* configurable category rules via TOML ([d293346](https://github.com/Dasperless/sortr/commit/d2933462ddbcd92500e64e0444369fc8cab2f596))
* move files safely with collision handling ([a682b06](https://github.com/Dasperless/sortr/commit/a682b06aeab7d2d8edc46ab4cc98d13ecf2ba2e1))
* record each run in a JSON journal ([d126820](https://github.com/Dasperless/sortr/commit/d126820d862ea0a65fde71cea5c96145cab24d9d))
* scan directories and plan moves by extension ([ae4f441](https://github.com/Dasperless/sortr/commit/ae4f44188f54b133028d2bef428ca3552ab95125))
