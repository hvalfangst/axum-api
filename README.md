# REST API written in Rust with Axum

## Overview



## Requirements

* x86-64
* Linux/Unix
* [Rust](https://www.rust-lang.org/tools/install)
* [Docker](https://www.docker.com/products/docker-desktop/)

## Startup

The script "up" starts the application by executing the following:
```
1. docker-compose -f db/docker-compose.yml up -d
2. cargo install diesel_cli --no-default-features --features "postgres"
3. disel setup
4. diesel migration run
5. cargo build
6. cargo run
```


## Usage

## Shutdown

The script "down" wipes the database by executing the following:
```
docker-compose -f db/docker-compose.yml down
```

## Postman Collection

The repository includes a Postman collection in the 'postman' directory.