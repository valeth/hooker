ARG rust_version="1.48"

FROM rust:${rust_version}-alpine AS build
RUN apk add --no-cache musl-dev
RUN mkdir -p /build
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY ./ ./
ARG build_target="release"
RUN test ${build_target} = "release" \
 && cargo build --release \
 || cargo build


FROM rust:${rust_version}-alpine
LABEL maintainer="Patrick Auernig <dev.patrick.auernig@gmail.com>"
ARG user_uid=1000
ARG user_gid=1000
RUN addgroup -S -g "$user_gid" app \
 && adduser -S -G app -u "$user_uid" app \
 && mkdir -p /app \
 && chown app:app /app
WORKDIR /app
USER app
ARG build_target="release"
COPY --chown=app:app --from=build /build/target/${build_target}/hooker ./hooker
EXPOSE 9292
CMD ["./hooker"]
