SHELL := /usr/bin/env bash

.PHONY: all run install deps

all: install deps
	cargo build

run: install deps
	cargo tauri dev

install:
	command -v cargo-tauri || (cargo install dioxus-cli && cargo install tauri-cli --version "^2")

deps:
	cargo fetch
