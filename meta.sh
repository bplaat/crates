#!/bin/bash
set -eo pipefail
exec cargo xtask "$@"
