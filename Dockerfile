FROM ekidd/rust-musl-builder as builder

ADD --chown=rust:rust . ./

RUN cargo build --release

FROM alpine

WORKDIR /bin

COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/cgf .
RUN chmod +x cgf

CMD ["cgf"]
