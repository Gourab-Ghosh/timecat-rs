# Use a specific base image
FROM rust:1-bullseye as builder

# Setup the working directory
WORKDIR /root/timecat

# Copy only the necessary files
COPY src ./src
COPY Cargo.toml .
COPY build.rs .
COPY README.md .
COPY documentation ./documentation

# Set environment variables to optimize build
ENV RUSTFLAGS="-C target-cpu=native"

# Build the application
RUN cargo build --release --bins

# Use a minimal base image for the final stage
FROM debian:bullseye-slim

# Copy the built executable from the builder stage
COPY --from=builder /root/timecat/target/release/timecat /usr/local/bin/timecat

# Set up runtime command
CMD ["/usr/local/bin/timecat", "--no-color", "--uci"]
