# Stage 1: Builder
FROM rust:latest as builder

# Install the musl target for static linking
RUN rustup target add x86_64-unknown-linux-musl

# Set the working directory inside the container
WORKDIR /usr/src/redisbrute


# Copy the Cargo.toml and Cargo.lock files to the working directory
COPY ./Cargo.toml ./Cargo.lock ./

# Create an empty Rust project to cache dependencies
RUN cargo new --bin redisbrute
WORKDIR /usr/src/redisbrute/

# Copy the source files into the container
COPY ./src ./src

# Build the project in release mode using musl for static linking
RUN cargo build --release --target x86_64-unknown-linux-musl

# Stage 2: Executor
FROM alpine:latest

# Set the maintainer of the image
LABEL maintainer="https://t.me/robensive"
# Install necessary dependencies for running the binary
RUN apk add --no-cache ca-certificates

# Set the working directory inside the container
WORKDIR /usr/local/bin

# Copy the statically linked binary from the builder stage
COPY --from=builder /usr/src/redisbrute/target/x86_64-unknown-linux-musl/release/redisbrute .

# Set the entrypoint to the compiled binary
RUN ls -la /usr/local/bin/redisbrute
ENTRYPOINT ["/usr/local/bin/redisbrute"]

# Set default command to show help
CMD ["--help"]
