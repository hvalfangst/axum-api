#!/bin/bash

# PostgreSQL container information
DB_CONTAINER_NAME="test_postgres_1"
DB_NAME="test_db"
DB_USER="Glossy"
DB_PASSWORD="yellau"

# Function to delete entries and measure time taken
delete_entries() {
    start_time=$(date +%s)
    docker exec -i "$DB_CONTAINER_NAME" psql -U "$DB_USER" -d "$DB_NAME" -c "DELETE FROM $1;"
    end_time=$(date +%s)
    elapsed_time=$((end_time - start_time))
    echo "Deleted entries from $1 in $((elapsed_time * 1000)) milliseconds"
}

# Delete entries from different tables and measure time
delete_entries "ships"
delete_entries "empires"
delete_entries "locations"

echo "All entries in the tables have been deleted."
