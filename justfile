#!/usr/bin/env just --justfile

set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

default:
    @just --list

# cargo build --example egui
examples-egui:
	cargo build --example egui

# cargo build --example egui-menu-events
examples-egui-menu:
	cargo build --example egui-menu-events

# Check cargo duplicate dependencies
dup:
    cargo tree -d

# Remove artifacts that cargo has generated
clean:
	cargo clean

# Count the lines of codes of this project
loc:
	@echo "--- Counting lines of .rs files (LOC):" && find src -type f -name "*.rs" -not -path "*/target/*" -exec cat {} \; | wc -l
