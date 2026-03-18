# Fix: Dependency updates

## Problem

`cargo install` reports newer major versions available:
- `colored` v2.2.0 → v3.1.1
- `notify` v7.0.0 → v8.2.0

## TODO

1. Review `colored` v3 changelog — check for breaking API changes
2. Review `notify` v8 changelog — check for breaking API changes (watcher setup, event types)
3. Update Cargo.toml and fix any compilation errors
4. Run all tests

## Phase

Low priority. Current versions work fine.
