#!/bin/bash

SUFFIX=""
if [[ "$(uname -m)" == "arm64" ]]; then
  SUFFIX="-arm64"
fi

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer$SUFFIX:0.14.0
