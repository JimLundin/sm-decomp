#!/usr/bin/env bash
# Install the independent sender offer inputs and following-entry queues.
set -euo pipefail
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" sender-offers-v1 \
  m0-sender-offers-v1-73a1d3e7c621b740.tar.gz.enc \
  73a1d3e7c621b740e909895a8ada33f2dd2fed527deda9c06e832c9809ac3a34 1728 2
