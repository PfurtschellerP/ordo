# Build environment of the fuzz targets (ADR-0059): OSS-Fuzz's Rust builder with nightly
# Rust and cargo-fuzz. Pinned by digest; Dependabot keeps it current.
FROM gcr.io/oss-fuzz-base/base-builder-rust:v1@sha256:3428d0a90b1e7efbe2ba947bd1d0c9a70594986b4b6553804547f1fb24cfc92f
RUN apt-get update -qq && apt-get install -y -qq zip >/dev/null && rm -rf /var/lib/apt/lists/*
COPY . $SRC/ordo
WORKDIR $SRC/ordo
COPY .clusterfuzzlite/build.sh $SRC/
