## builder
FROM alpine:edge AS builder

# show backtraces
ENV RUST_BACKTRACE 1

RUN apk add --update --no-cache \
  alpine-sdk \
  cargo \
  rust \
  cmake \
  openssl-dev

WORKDIR /etherinit

COPY . .

RUN cargo build --release --target x86_64-alpine-linux-musl

# copy binary to /usr/bin
RUN cp target/x86_64-alpine-linux-musl/release/etherinit /usr/bin/etherinit

## etherinit
FROM alpine:edge

# show backtraces
ENV RUST_BACKTRACE 1

RUN apk add --update --no-cache \
  libstdc++ \
  libgcc \
  openssl

COPY --from=builder /usr/bin/etherinit /usr/bin/etherinit

ENTRYPOINT [ "etherinit", "version" ]
