# Contributing to Vega

Thank you for your interest in contributing to Vega! This document provides guidelines and instructions for contributing.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/<you>/vega.git`
3. Create a feature branch: `git checkout -b feature/my-feature`
4. Make your changes
5. Run tests: `cargo test --workspace`
6. Run lints: `cargo clippy --workspace`
7. Format code: `cargo fmt --all`
8. Commit and push
9. Open a Pull Request

## Development Setup

```bash
# Install Rust (stable)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/AarambhDevHub/vega.git
cd vega
cargo build --workspace

# Run tests
cargo test --workspace

# Run the demo app
cargo run -p blog-auth-demo

# List discovered routes
cargo run -p vega-cli -- routes --path examples/blog-auth-demo
```

## Project Structure

```
vega/
├── crates/
│   ├── vega-core/      # Core types, enums, traits
│   ├── vega-config/    # Vega.toml parsing
│   ├── vega-router/    # File-based route scanner + codegen
│   ├── vega-macros/    # Proc macros (#[page], #[get], etc.)
│   ├── vega-server/    # Axum integration, SSR, API handling
│   ├── vega-client/    # WASM/client runtime (planned)
│   ├── vega-fetch/     # Data fetching helpers
│   ├── vega-cli/       # CLI binary (vega new, dev, build, etc.)
│   └── vega/           # Facade crate (re-exports everything)
├── examples/
│   └── blog-auth-demo/ # Full demo application
└── docs/               # Architecture and design docs
```

## Coding Guidelines

- **Rust edition**: 2021
- **Formatting**: `cargo fmt --all` before committing
- **Linting**: `cargo clippy --workspace` with zero warnings
- **Tests**: Add tests for new functionality
- **Documentation**: Add `///` doc comments to all public items
- **Error handling**: Use `thiserror` for library errors, `anyhow` in binaries
- **No `unwrap()` in library code**: Use proper error propagation

## Commit Messages

Use conventional commit format:
```
feat(router): add catch-all route support
fix(server): correct cookie parsing for empty values
docs(readme): add quick start guide
test(fetch): add search param merge tests
```

## Pull Request Process

1. Update documentation for any API changes
2. Add tests for new features
3. Ensure CI passes (check, test, clippy, fmt)
4. Update CHANGELOG.md
5. Request review from a maintainer

## Reporting Issues

- Use GitHub Issues
- Include Rust version (`rustc --version`)
- Include OS and architecture
- Provide minimal reproduction steps
- Include relevant error messages

## License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license.
