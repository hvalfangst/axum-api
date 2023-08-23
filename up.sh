#!/bin/sh

# Exits immediately if a command exits with a non-zero status
set -e

# Run 'docker-compose up' for source database deployment
docker-compose -f db/docker-compose.yml up -d

disel setup

diesel migration run

cargo build

cargo run




