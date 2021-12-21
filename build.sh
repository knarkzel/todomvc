#!/bin/sh
cargo build --release
cp target/release/todomvc .
strip todomvc
upx --best --lzma todomvc
ls -alh todomvc
