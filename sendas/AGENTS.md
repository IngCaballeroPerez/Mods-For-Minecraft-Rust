# Working on Sendas, a native Horizon mod

Read README.md and compatibility.json before editing. This folder is one mod of
the repository; each mod keeps to its own folder. Keep Horizon and
horizon-mixins clones outside the repository. Only export committed pinned
Horizon sources into ignored target trees; never modify the original checkout
or player resources. Do not commit binaries, exported sources or Minecraft
assets.

The Cargo library and injected runtime module share `mixins/sendas/runtime.rs`.
Keep helper functions small and gameplay logic testable there, with the
standard library only. All AST changes go through horizon-mixins plan/apply; do
not add string-based Rust source rewriting.

Player-facing text and comments are in Spanish; identifiers are in English.
Names, rules, numbers and text are the project's own: copy nothing from other
mods or modpacks.

Validate with cargo fmt, clippy with warnings denied and cargo test. For
bundle/compatibility changes also run python3 tools/test_stage.py and
python3 tools/horizon.py check/test against the pinned source export. Keep
validation claims and supported revisions honest: VALIDATION.md says only what
was actually run.
