#!/bin/sh

# Exits immediately if a command exits with a non-zero status
set -e

# Run 'docker-compose up' for creating databases used in the application
docker-compose -f db/dev/docker-compose.yml up -d
docker-compose -f db/test/docker-compose.yml up -d

cargo install diesel_cli --no-default-features --features "postgres"

# Run migrations for our development database
diesel migration run --database-url="postgres://Tremakken:yeah???@localhost:3333/dev_db"

# Run migrations for our test database
diesel migration run --database-url="postgres://Glossy:yellau@localhost:4444/test_db"

# Compiles the application
cargo build

# Runs the tests single-threaded in order to avoid connection pool race conditions
cargo test -- --test-threads=1

# Serves the exposed endpoints with Axum via underlying Hyper layer
cargo run