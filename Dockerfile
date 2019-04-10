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

WORKDIR /fst-etherinit

COPY . .

RUN cargo build --release --target x86_64-alpine-linux-musl

# copy binary to /usr/bin
RUN cp target/x86_64-alpine-linux-musl/release/fst-etherinit /usr/bin/fst-etherinit

## fst-etherinit
FROM alpine:edge

# show backtraces
ENV RUST_BACKTRACE 1

RUN apk add --update --no-cache \
  libstdc++ \
  libgcc \
  openssl

COPY --from=builder /usr/bin/fst-etherinit /usr/bin/fst-etherinit

ENTRYPOINT [ "fst-etherinit", "version" ]
