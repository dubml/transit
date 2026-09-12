FROM rust:1.88-bookworm AS builder

WORKDIR /workspace
COPY . .
RUN rm -f toolchain.toml && cargo build --release --bin transit

FROM debian:bookworm-slim

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /workspace/target/release/transit /usr/local/bin/transit

USER 65532:65532
EXPOSE 26080 26443 26021
ENTRYPOINT ["/usr/local/bin/transit"]
