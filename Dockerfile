
FROM rust:1.91 AS builder
WORKDIR /usr/src/myapp
# For now we use only one Dockerfile for both the server and the agent, because it's easier.
# We could make 2 image to remove the unused binary.

# this allow to cache rust deps build.
RUN mkdir -p  src/bin && echo "fn main() {}" > src/bin/dummy.rs
COPY Cargo.toml .
RUN cargo build --release
RUN rm src/bin/dummy.rs
# here we copy our real code and install it in the filesystem
COPY . .
RUN cargo install --path .

FROM debian:trixie-slim
# dev debug deps :
RUN apt update && apt install iputils-ping nmap -y

# copy the executable we need in the debian small image, to avoid having all rust deps (2GB) in final image.
COPY --from=builder /usr/local/cargo/bin/server /usr/local/bin/server
COPY --from=builder /usr/local/cargo/bin/agent /usr/local/bin/agent
EXPOSE 3000
# by default we run the server, see docker-compose-rust.yaml for agent mode.
ENTRYPOINT ["server"]
