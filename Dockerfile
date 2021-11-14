# Rust as the base image
FROM rust:1.49 as build

# 1. Create a new empty shell project
RUN USER=root cargo new --lib butter
WORKDIR /butter

# 2. Copy our manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# 3. Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# 4. Now that the dependency is built, copy your source code
COPY ./src ./src

# 5. Build for release
RUN rm ./target/release/deps/butter*
RUN cargo install --path .

# our final base
FROM debian:buster-slim

# copy the build artifact from the build stage
COPY --from=build /butter/target/release/main .