FROM ubuntu:latest
LABEL authors="derdilla"

# build binaries
FROM rust:1.94-alpine AS build-base

WORKDIR /app

RUN rustup target add x86_64-unknown-linux-musl

ENV CARGO_BUILD_BUILD_DIR=/app/build
ENV CARGO_TARGET_DIR=/app/target

COPY --from=fireframe . fireframe

FROM build-base AS build-fireframe
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=cache,target=/app/build/ \
    cd fireframe && cargo build --locked --release --target=x86_64-unknown-linux-musl --bin init


FROM build-fireframe AS build-textury
COPY --from=textury . textury

RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=cache,target=/app/build/ \
    cd textury && cargo build --locked --release --target=x86_64-unknown-linux-musl --bin vsock_server

# build runtime
FROM alpine:3.23 AS runtime

################################################################################
# Setup compilers: inspired by: https://github.com/pandoc/dockerfiles/blob/main/3.9/alpine/latex/Dockerfile
################################################################################

# Install typst
RUN apk --no-cache add typst

# Install Texlive
ARG TL_MIRROR="https://texlive.info/CTAN/systems/texlive/tlnet"

RUN apk add --no-cache perl curl fontconfig libgcc gnupg && \
    mkdir "/tmp/texlive" && cd "/tmp/texlive" && \
    wget "$TL_MIRROR/install-tl-unx.tar.gz" && \
    tar xzvf ./install-tl-unx.tar.gz && \
    ( \
        echo "selected_scheme scheme-minimal" && \
        echo "instopt_adjustpath 0" && \
        echo "tlpdbopt_install_docfiles 0" && \
        echo "tlpdbopt_install_srcfiles 0" && \
        echo "TEXDIR /opt/texlive/" && \
        echo "TEXMFLOCAL /opt/texlive/texmf-local" && \
        echo "TEXMFSYSCONFIG /opt/texlive/texmf-config" && \
        echo "TEXMFSYSVAR /opt/texlive/texmf-var" && \
        echo "TEXMFHOME ~/.texmf" \
    ) > "/tmp/texlive.profile" && \
    "./install-tl-"*"/install-tl" --location "$TL_MIRROR" -profile "/tmp/texlive.profile" && \
    rm -vf "/opt/texlive/install-tl" && \
    rm -vf "/opt/texlive/install-tl.log" && \
    rm -vrf /tmp/*

ENV PATH="${PATH}:/opt/texlive/bin/x86_64-linuxmusl"

ARG TL_SCHEME_BASIC="y"
RUN if [ "$TL_SCHEME_BASIC" = "y" ]; then tlmgr install scheme-basic; fi

ARG TL_SCHEME_SMALL="y"
RUN if [ "$TL_SCHEME_SMALL" = "y" ]; then tlmgr install scheme-small; fi

ARG TL_SCHEME_MEDIUM="y"
RUN if [ "$TL_SCHEME_MEDIUM" = "y" ]; then tlmgr install scheme-medium; fi

ARG TL_SCHEME_FULL="n"
RUN if [ "$TL_SCHEME_FULL" = "y" ]; then tlmgr install scheme-full; fi


RUN adduser -H -S -u 868 app

COPY --from=build-fireframe /app/target/x86_64-unknown-linux-musl/release/init /init
COPY --from=build-textury /app/target/x86_64-unknown-linux-musl/release/vsock_server /bin/app
