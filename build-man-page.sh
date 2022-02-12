#!/bin/sh
set -e -u

pandoc man/rust-script.md -s -t man -o target/rust-script.1
