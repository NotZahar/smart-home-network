#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(dirname -- "${BASH_SOURCE[0]}")"

cd "$SCRIPT_DIR"

BUILD_MODE="debug"
BUILD_FLAG=""
REMOVE_TARGET=true

while [[ $# -gt 0 ]]; do
	case $1 in
	-r | --release)
		BUILD_MODE="release"
		BUILD_FLAG="--release"
		shift
		;;
	-d | --debug)
		BUILD_MODE="debug"
		BUILD_FLAG=""
		shift
		;;
	-k | --keep-target)
		REMOVE_TARGET=false
		shift
		;;
	-h | --help)
		echo "Usage: $0 [OPTIONS]"
		echo "  -d, --debug        Build in debug mode (default)"
		echo "  -r, --release      Build in release mode"
		echo "  -k, --keep-target  Keep the target directory"
		echo "  -h, --help         Show this help message"
		exit 0
		;;
	*)
		echo "Error: Unknown option $1"
		exit 1
		;;
	esac
done

if $REMOVE_TARGET; then
	rm -rf target
fi

echo "Build type is ${BUILD_MODE}"

echo ""
echo "Running clippy ..."
cargo clippy ${BUILD_FLAG} --all-targets --all-features -- -D warnings

echo ""
echo "Checking formatting ..."
cargo fmt --check

echo ""
echo "Running tests ..."
cargo test ${BUILD_FLAG} --all-features

echo ""
echo "Building ..."
cargo build ${BUILD_FLAG} --jobs "$(nproc)"
