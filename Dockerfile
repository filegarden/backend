# syntax=docker/dockerfile:1

# We use Alpine Linux since it's lightweight (much smaller and faster to build).
FROM --platform=$BUILDPLATFORM rust:1.75.0-alpine AS build

WORKDIR /app

# Install needed build dependencies (since Alpine doesn't come with much).
RUN apk add --no-cache clang lld musl-dev git file

COPY . .

ARG BIN_NAME
ARG CARGO_ARGS=

# Persist directories with downloaded or compiled dependencies between builds so
# every build doesn't have to redownload and recompile all dependencies. Then
# build the binary package in release mode, and copy it out of the cache mount
# before it's unmounted after this step.
RUN --mount=type=cache,target=target \
    --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry \
    <<EOF
set -e
cargo build --package $BIN_NAME --locked $CARGO_ARGS
cp ./target/release/$BIN_NAME /bin/app
EOF

# The final image will be a lean Alpine instance with only the final binary from
# the above build image copied into it.
FROM alpine AS final

# Create a non-privileged user for the app to run under.
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "10001" \
    app
USER app

COPY --from=build /bin/app /bin/app

ENTRYPOINT ["/bin/app"]
