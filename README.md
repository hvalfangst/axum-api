# REST API written in Rust with Axum

## Overview



## Requirements

* x86-64
* Linux/Unix
* [Rust](https://www.rust-lang.org/tools/install)
* [Docker](https://www.docker.com/products/docker-desktop/)

## Startup

The script "up" provisions resources and starts our application by executing the following:
```
1. docker-compose -f db/dev/docker-compose.yml up -d
2. docker-compose -f db/test/docker-compose.yml up -d
3. cargo install diesel_cli --no-default-features --features "postgres"
4. diesel migration run --database-url="postgres://Tremakken:yeah???@localhost:3333/dev_db"
5. diesel migration run --database-url="postgres://Glossy:yellau@localhost:4444/test_db"
6. cargo build
7. cargo test -- --test-threads=1
8. cargo run
```


## Shutdown

The script "down" deletes our dev and test databases by executing the following:
```
1. docker-compose -f db/docker-compose.yml down
2. docker-compose -f db/test/docker-compose.yml down
```

## Test script
The script "test" resets db and executes tests by executing the following:
```
1. sh db/test/reset.sh
2. cargo test -- --test-threads=1
```

## Postman Collection

The repository includes a Postman collection in the 'postman' directory.