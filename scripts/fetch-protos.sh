#!/usr/bin/env bash
# Fetch conformance .proto files from the tools Docker image into
# conformance/protos/ for local development.
#
# Usage: scripts/fetch-protos.sh [tools-image-tag]
#   tools-image-tag defaults to ghcr.io/anthropics/buffa/tools:v33.5

set -euo pipefail

TOOLS_IMAGE="${1:-ghcr.io/anthropics/buffa/tools:v33.5}"
DEST="$(cd "$(dirname "$0")/.." && pwd)/conformance/protos"

echo "Pulling tools image: ${TOOLS_IMAGE}"
docker pull "${TOOLS_IMAGE}" 2>/dev/null || {
    echo "Could not pull image — building locally (this will take ~10 min)..."
    docker build \
        -t "${TOOLS_IMAGE}" \
        -f "$(dirname "$0")/../conformance/Dockerfile.tools" \
        "$(dirname "$0")/.."
}

echo "Extracting proto files to ${DEST}..."
# FROM scratch images have no default CMD/ENTRYPOINT so docker create requires
# a dummy command.  The container is never started — we only use `docker cp`.
CID=$(docker create "${TOOLS_IMAGE}" /dev/null)
trap "docker rm -f ${CID} >/dev/null 2>&1" EXIT

mkdir -p "${DEST}"
docker cp "${CID}:/protos/." "${DEST}/"

echo "Done. Proto files written to conformance/protos/"
