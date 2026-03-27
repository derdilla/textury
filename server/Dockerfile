# Make sure RUST_VERSION matches the Rust version
ARG RUST_VERSION=1.92

################################################################################
# Create a stage for building the application.
################################################################################

FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME
WORKDIR /app

# Build the application.

COPY . .
ENV CARGO_BUILD_BUILD_DIR=/app/build
ENV CARGO_TARGET_DIR=/app/target
RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=cache,target=/app/build/ \
      cargo build --locked --release

FROM alpine:3.23 AS final

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

################################################################################
# Minimal server setup
################################################################################

# Copy the executable from the "build" stage.
COPY --from=build /app/target/release/server /bin/server

# Create a non-privileged user that the app will run under.
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --shell "/sbin/nologin" \
    --home "/appdata" \
    --uid "${UID}" \
    appuser
USER appuser
WORKDIR /appdata

# Expose the port that the application listens on.
EXPOSE 3000

# What the container should run when it is started.
CMD ["/bin/server"]
