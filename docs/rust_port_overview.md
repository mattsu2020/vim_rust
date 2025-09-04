# Rust Port Overview

This document outlines where legacy C implementations exist for several features
and the corresponding Rust crates that provide replacements.

## Diff

- C source: `src/diff.c` – handles buffer diffing but delegates the diff engine to
  the Rust implementation via `rust_diff`.
- Rust crate: `rust_diff` – provides `xdl_diff`, unified/indices output, and
  line matching utilities.

## Spell Checking

- C source: `src/spell.c` – legacy spell checking logic that now calls into
  Rust for dictionary handling, word checking, and suggestions.
- Rust crate: `rust_spell` – exposes dictionary loading, word validation, and
  suggestion generation.

## Profiler

- C source: `src/profiler.c` – Vim script profiling still implemented in C.
- Rust crate: `rust_profiler` – offers basic timers and formatting; further
  integration is required to fully replace the C profiler.

## Sound

- C source: `src/sound.c` – thin wrappers around the Rust sound backend.
- Rust crate: `rust_sound` – plays audio files and events using the `rodio`
  library.

Each crate contains its own tests. As functionality is migrated, these C
sources can be reduced or removed.
