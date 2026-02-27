#!/usr/bin/env sh
set -eu
input="${1-}"
printf "%s" "$input" > .benchmark_input
exec ./app
