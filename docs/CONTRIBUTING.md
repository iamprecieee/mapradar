# Contributing

## Quick Start

```bash
git clone https://github.com/iamprecieee/mapradar
cd mapradar
uv venv
source .venv/bin/activate
uv add maturin
maturin develop
```

## Development

| Command | Purpose |
|---------|---------|
| `cargo build` | Compile Rust code |
| `cargo test` | Run Rust unit tests |
| `cargo clippy` | Lint Rust code |
| `cargo fmt` | Format Rust code |
| `maturin develop` | Build and install Python bindings |

## Pull Requests

1. Fork and create a feature branch
2. Follow existing code style
3. Add tests for new functionality
4. Ensure all checks pass
5. Update documentation if needed

## Code Style

- Use `cargo fmt` before committing
- Handle errors explicitly with `Result` types
- Add doc comments for public APIs
- Keep functions focused and under 40 lines

## Commit Messages

Format: `type(scope): description`

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## License

Contributions licensed under [MIT](../LICENSE).
