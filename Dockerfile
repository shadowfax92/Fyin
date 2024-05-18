# Use the official Rust image as the base
FROM rust:latest

# Set the working directory inside the container
WORKDIR /usr/src/fyin

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml ./

# Copy the source code
COPY src ./src

# Build the project
RUN cargo build --release

# Copy the built binary to the final image
RUN mkdir -p /usr/local/bin && cp target/release/fyin /usr/local/bin/fyin

# Define the entrypoint to run your CLI application
ENTRYPOINT ["fyin"]


