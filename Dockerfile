FROM gcr.io/distroless/static:nonroot
COPY ./target/x86_64-unknown-linux-musl/release/p2p-rs ./p2p-rs
ENV RUSTLOG=DEBUG
ENTRYPOINT [ "./p2p-rs" ]
