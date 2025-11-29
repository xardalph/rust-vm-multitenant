
FROM rust:1.91 AS builder
WORKDIR /usr/src/agent
# For now we use only one Dockerfile for both the server and the agent, because it's easier.
# We could make 2 image to remove the unused binary.

# This allow to cache rust deps build.
RUN mkdir -p  src/bin && echo "fn main() {}" > src/bin/dummy.rs
COPY Cargo.toml .
RUN cargo build --release
RUN rm src/bin/dummy.rs

# Here we copy our real code and install it in the filesystem.
COPY . .
RUN cargo install --path .

# We create a small image with only the compiled binary.
FROM debian:trixie-slim
RUN apt update

# Some debug utils.
RUN apt install iputils-ping nmap -y

# Copy the executable we need in the debian small image, to avoid having all rust deps (2GB) in final image.
COPY --from=builder /usr/local/cargo/bin/agent /usr/local/bin/agent

ENTRYPOINT ["agent"]
