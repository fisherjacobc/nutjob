# Builder
FROM rust AS builder

# Set working directory
WORKDIR /usr/src/nutjob

# Copy everything (source and manifest)
COPY . .

# Build in release mode
RUN rustup target add aarch64-unknown-linux-musl
RUN cargo build --release --target aarch64-unknown-linux-musl

# Runtime img
FROM ubuntu

# Get ping util
RUN apt update 
RUN apt install -y iputils-ping

# Create nutjob working directory
RUN mkdir /nutjob
WORKDIR /nutjob

# Copy built binary
COPY --from=builder /usr/src/nutjob/target/aarch64-unknown-linux-musl/release/nutjob /nutjob/nutjob-bin

# Run the binary
CMD ["./nutjob-bin"]
