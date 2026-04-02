# Contributing to rust2d_ui

Thank you for considering a contribution! This document explains how to
participate effectively.

## Getting started

```bash
git clone https://github.com/harupipipipi/rust-GL.git
cd rust-GL
cargo check --lib
cargo test
```

## Branching & pull requests

1. Fork the repository and create a branch from `main`.
2. Name your branch descriptively: `fix/canvas-clip`, `feat/gradient-fill`, etc.
3. Keep each PR focused on **one** logical change.
4. Ensure `cargo test`, `cargo clippy -- -D warnings`, and `cargo fmt --check`
   all pass before opening the PR.
5. Fill in the pull-request template that appears automatically.

## Coding conventions

- **Edition**: Rust 2021.
- **Formatting**: `rustfmt` defaults (run `cargo fmt`).
- **Lints**: The CI runs Clippy with `-D warnings`. Fix all warnings before
  pushing.
- **Error handling**: Use `thiserror` for library errors; avoid `.unwrap()` in
  library code.
- **Tests**: Add or update tests for every behavioral change. Tests live in
  `src/lib.rs` (unit) or future `tests/` (integration).
- **Documentation**: Public items must have `///` doc-comments.

## AI-generated code policy

This project was bootstrapped with AI assistance and welcomes AI-generated
contributions under the following rules:

1. **Disclosure**: Mark the PR with the label `ai-generated` or state it in the
   description.
2. **Human review required**: Every AI-generated diff must be reviewed and
   understood by a human before merge.
3. **Test coverage**: AI-generated code must include tests. Untested AI code
   will not be merged.
4. **License compliance**: You must verify that the AI output does not
   introduce code with an incompatible license.

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/) style:

```
feat(canvas): add gradient fill support
fix(layout): correct padding calculation for nested containers
docs: update CONTRIBUTING with AI policy
ci: add macOS job to cross-platform workflow
```

## Reporting bugs

Use the **Bug Report** issue template. Include Rust version (`rustc --version`),
OS, and a minimal reproduction.

## Suggesting features

Use the **Feature Request** issue template. Describe the use-case before
proposing a solution.

## License

By contributing you agree that your contributions are licensed under the
[MIT License](LICENSE).
