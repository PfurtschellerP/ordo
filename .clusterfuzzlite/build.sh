#!/bin/bash -eu
# Builds the fuzz targets and their seed corpora (ADR-0059) into $OUT.
cd "$SRC/ordo/tests/fuzz"
cargo fuzz build -O --fuzz-dir .
targets="client_message server_message signed_event payloads"
for target in $targets; do
  cp "target/x86_64-unknown-linux-gnu/release/$target" "$OUT/"
done
# Seeds from real values, built without the fuzzing instrumentation.
env -u RUSTFLAGS -u CFLAGS -u CXXFLAGS cargo run --release --bin seeds -- "$WORK/corpus"
for target in $targets; do
  (cd "$WORK/corpus/$target" && zip -q "$OUT/${target}_seed_corpus.zip" ./*)
done
