#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(dirname -- "${BASH_SOURCE[0]}")"

cd "$SCRIPT_DIR"

BUILD_MODE="debug"
APP_ARGS=()

while [[ $# -gt 0 ]]; do
	case $1 in
	-r | --release)
		BUILD_MODE="release"
		shift
		;;
	-d | --debug)
		BUILD_MODE="debug"
		shift
		;;
	-h | --help)
		echo "Usage: $0 [OPTIONS] [-- APP_ARGS...]"
		echo "Options:"
		echo "  -r, --release  Run in release mode"
		echo "  -d, --debug    Run in debug mode (default)"
		echo "  -h, --help     Show this help message"
		echo "Example:"
		echo "  $0 -- --start-simulators"
		exit 0
		;;
	--)
		shift
		APP_ARGS+=("$@")
		break
		;;
	*)
		APP_ARGS+=("$1")
		shift
		;;
	esac
done

BINARY_PATH="target/$BUILD_MODE/smart-home-network"

if [[ ! -x "$BINARY_PATH" ]]; then
	echo "Error: binary not found at $BINARY_PATH"
	exit 1
fi

"$BINARY_PATH" "${APP_ARGS[@]}"
