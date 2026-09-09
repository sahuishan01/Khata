FROM docker.io/library/rust:1-bookworm AS chef
RUN cargo install cargo-chef
WORKDIR /build

FROM chef AS planner
COPY backend/ .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Download a pinned pdfium prebuilt (loaded dynamically at runtime; not bundled).
# Kept above `COPY backend/ .` so a source-only change does not re-download it.
ARG PDFIUM_VER=chromium/8044
RUN set -eux; \
    deb_arch="$(dpkg --print-architecture)"; \
    case "$deb_arch" in \
      amd64) pdfium_arch=x64 ;; \
      arm64) pdfium_arch=arm64 ;; \
      *) echo "unsupported arch: $deb_arch" >&2; exit 1 ;; \
    esac; \
    mkdir -p /pdfium; \
    curl -fsSL "https://github.com/bblanchon/pdfium-binaries/releases/download/${PDFIUM_VER}/pdfium-linux-${pdfium_arch}.tgz" \
      | tar -xz -C /pdfium

COPY backend/ .
RUN cargo build --release

FROM docker.io/library/debian:bookworm-slim
RUN apt-get update -qq && apt-get install -y -qq ca-certificates libgcc-s1 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/khata /usr/local/bin/khata
COPY --from=builder /pdfium/lib/libpdfium.so /usr/local/lib/libpdfium.so
ENV PDFIUM_LIB_PATH=/usr/local/lib/libpdfium.so
RUN ldconfig
EXPOSE 8090
CMD ["khata"]
