# Use the official Rust image (Debian-based Linux)
FROM rust:latest

# Install the 9P client tools (so we can test the mount command)
RUN apt-get update && apt-get install -y \
    diod \
    net-tools \
    iproute2

# Set up the workspace
WORKDIR /usr/src/willowd
COPY . .

# Build the daemon
RUN cargo build --release

# Expose the 9P port
EXPOSE 5640

# Default command: Run the daemon
CMD ["./target/release/willowd"]
