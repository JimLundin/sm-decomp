#!/usr/bin/env bash
# Reviewed ordinary power shot, contact and flying picture insertion.
set -euo pipefail
if [ "$#" -ne 2 ]; then
  echo "usage: $0 <rom> <existing-native-fixture-dir>" >&2
  exit 2
fi
bash "$(dirname "$0")/fetch_native_fixture.sh" "$1" "$2" shot-contact-v1 \
  m0-shot-contact-fixture-1b92b153e964bb53.tar.gz.enc \
  1b92b153e964bb53c0294aa5440400cf7365898bcfe4c664eb1c0a9b03e340c3 1536 2
echo 'SHOT CONTACT FIXTURE PIN OK'
