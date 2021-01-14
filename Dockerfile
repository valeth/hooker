ARG rust_version="1.49"
ARG alpine_version="3.12"

FROM rust:${rust_version}-alpine${alpine_version} AS build
RUN apk add --no-cache musl-dev
RUN mkdir -p /build
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY ./ ./
ARG build_target="release"
RUN test ${build_target} = "release" \
 && cargo build --release \
 || cargo build


FROM alpine:${alpine_version}
LABEL maintainer="Patrick Auernig <dev.patrick.auernig@gmail.com>"
RUN apk add --no-cache ca-certificates
ARG user_uid=1000
ARG user_gid=1000
RUN addgroup -S -g "$user_gid" app \
 && adduser -S -G app -u "$user_uid" app \
 && mkdir -p /app /app/data \
 && chown -R app:app /app
WORKDIR /app
USER app
ARG build_target="release"
COPY --chown=app:app --from=build /build/target/${build_target}/hooker ./hooker
EXPOSE 9292
ENTRYPOINT ["./hooker"]
