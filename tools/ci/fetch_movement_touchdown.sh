#!/usr/bin/env bash
# Install native touchdown inputs and independent observations from the locked room.
set -euo pipefail
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" movement-touchdown-v1 \
  movement-touchdown-v1-e4766ec3cec50c46.tar.gz.enc \
  e4766ec3cec50c460d05828ed75ee053c8509071ab665bf9a69d36b0db9c9ba6 1600 2
