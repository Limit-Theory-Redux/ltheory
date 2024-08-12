#!/bin/bash
set -e

# Usage: ./expand.sh input::device

cargo expand -p phx $1 > expand.rs
