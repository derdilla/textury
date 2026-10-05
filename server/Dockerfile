# Make sure RUST_VERSION matches the Rust version
ARG RUST_VERSION=1.98

################################################################################
# Create a stage for building the application.
################################################################################

FROM rust:${RUST_VERSION}-alpine AS build
WORKDIR /app

# Build the application.

ENV CARGO_BUILD_BUILD_DIR=/app/build
ENV CARGO_TARGET_DIR=/app/target

COPY . .

RUN --mount=type=cache,target=/usr/local/cargo/git/db \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    --mount=type=cache,target=/app/build/ \
      cargo build --locked --release --bin server --features alpine

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

ARG TL_SCHEME_FULL="N"
RUN if [ "$TL_SCHEME_FULL" = "y" ]; then tlmgr install scheme-full; fi

# install bubblewrap
RUN apk add bubblewrap

################################################################################
# Minimal server setup
################################################################################

# Create a non-privileged user that the app will run under.
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --shell "/sbin/nologin" \
    --home "/appdata" \
    --uid "${UID}" \
    appuser
RUN chown -R appuser:appuser /tmp
USER appuser
WORKDIR /appdata

RUN mkdir -p /appdata/projects

# Expose the port that the application listens on.
EXPOSE 3000

# Copy the executable from the "build" stage last for performance.
COPY --from=build /app/textury.toml /bin/textury.toml
COPY --from=build /app/target/release/server /bin/server

# What the container should run when it is started.
CMD ["/bin/server", "--config-path", "/bin/textury.toml"]
