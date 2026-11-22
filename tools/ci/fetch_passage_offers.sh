#!/usr/bin/env bash
# Install the touched doorway and native pre-service passage sound markers.
set -euo pipefail
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" passage-offers-v1 \
  m0-passage-offers-v1-b7e4812d19057ca5.tar.gz.enc \
  b7e4812d19057ca5d0d25fbf996526fedd3896b50889ae2b5cbc3b3f7f8e96d4 193424 7
