#!/bin/sh
patch src/main.rs ssbc.rs.patch -o ssbc.rs
rustc --edition 2018 ssbc.rs $@
