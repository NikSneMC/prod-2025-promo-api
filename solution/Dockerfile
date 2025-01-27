FROM rust:1.84.0-alpine AS build
ENV PKG_CONFIG_ALLOW_CROSS=1

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

WORKDIR /usr/src/solution
COPY ./solution .
RUN cargo build --release

FROM alpine:latest

RUN apk add --no-cache ca-certificates openssl

COPY --from=build /usr/src/solution/target/release/solution /solution/solution
COPY --from=build /usr/src/solution/migrations/* /solution/migrations/
COPY --from=build /usr/src/solution/assets /solution/assets

WORKDIR /solution

# Run the application
CMD ["/solution/solution"]
