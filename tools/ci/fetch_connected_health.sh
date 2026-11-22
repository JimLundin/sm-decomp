#!/usr/bin/env bash
# Install the common-origin Health and HUD replay profiles and independent witnesses.
set -euo pipefail
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" connected-health-v1 \
  connected-health-v1-3282575811dbda9f.tar.gz.enc \
  3282575811dbda9f802631a5d563d4edeeb3a8d0f4e6cc356a9c328180377add 516688 32
