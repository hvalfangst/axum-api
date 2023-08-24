#!/bin/sh

# Exits immediately if a command exits with a non-zero status
set -e

# Run 'docker-compose up' for source database deployment
docker-compose -f db/docker-compose.yml up -d

# Install CLI associated with Diesel crate
cargo install diesel_cli --no-default-features --features "postgres"

# Connect to database
diesel setup

# Run the "up" portion of migration files located under folder "migrations"
diesel migration run

# Compiles the application
cargo build

# Serves the exposed endpoints with Axum via underlying Hyper layer
cargo run




