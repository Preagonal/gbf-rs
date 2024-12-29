#!/usr/bin/env bash

mkdir -p target/coverage
cargo llvm-cov --branch --all-features --workspace --cobertura --output-path target/coverage/cobertura.xml