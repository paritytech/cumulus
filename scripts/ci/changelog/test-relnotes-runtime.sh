export RUSTC_STABLE="rustc 1.56.1 (59eed8a2a 2021-11-01)"
export RUSTC_NIGHTLY="rustc 1.57.0-nightly (51e514c0f 2021-09-12)"
export PRE_RELEASE=true
export HIDE_SRTOOL_ROCOCO=true
export HIDE_SRTOOL_SHELL=true
export REF1=statemine-v5.0.0
export REF2=HEAD
export DEBUG=1
export NO_CACHE=1
export RELEASE_TYPE=

tera --env --env-key env --include-path templates --template templates/template.md.tera test/runtime-context.json
