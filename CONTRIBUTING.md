# Contributing to TrueNAS Exporter

Thank you for your interest in contributing to TrueNAS Exporter! This document outlines our contribution workflow and guidelines.

## Branching Strategy

We follow a **Gitflow-inspired workflow** with two main branches:

- **`main`**: Production-ready releases only
- **`develop`**: Integration branch for features and fixes

### Branch Protection

Both `main` and `develop` are **protected branches**:

- ❌ No direct commits allowed
- ❌ No force pushes allowed
- ✅ All changes must go through Pull Requests
- ✅ All CI checks must pass before merging

## Contribution Workflow

### 1. Fork the Repository

Click the "Fork" button on GitHub to create your own copy of the repository.

### 2. Clone Your Fork

```bash
git clone https://github.com/mlgruby/truenas-exporter-rs.git
cd truenas-exporter-rs
```

### 3. Add Upstream Remote

```bash
git remote add upstream https://github.com/mlgruby/truenas-exporter-rs.git
```

### 4. Create a Feature Branch

**Always branch from `develop`:**

```bash
git checkout develop
git pull upstream develop
git checkout -b feature/your-feature-name
```

**Branch naming conventions:**

- `feature/` - New features (e.g., `feature/iscsi-metrics`)
- `fix/` - Bug fixes (e.g., `fix/websocket-reconnect`)
- `docs/` - Documentation updates (e.g., `docs/grafana-dashboard`)
- `refactor/` - Code refactoring (e.g., `refactor/metrics-module`)

### 5. Make Your Changes

- Write clean, well-documented code
- Follow Rust best practices
- Add tests for new functionality
- Update documentation as needed

### 6. Commit Your Changes

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```bash
git commit -m "feat: add iSCSI target metrics"
git commit -m "fix: resolve WebSocket connection timeout"
git commit -m "docs: add Grafana dashboard examples"
```

**Commit message format:**

```text
<type>: <description>

[optional body]

[optional footer]
```

**Types:**

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `perf`: Performance improvements

### 7. Push to Your Fork

```bash
git push origin feature/your-feature-name
```

### 8. Create a Pull Request

1. Go to your fork on GitHub
2. Click "New Pull Request"
3. **Set base branch to `develop`** (NOT `main`)
4. Fill out the PR template with:
   - Description of changes
   - Related issue numbers (if applicable)
   - Testing performed
   - Breaking changes (if any)

## Pull Request Guidelines

### PR Requirements

- ✅ All CI checks must pass
- ✅ Code must be formatted (`cargo fmt`)
- ✅ No clippy warnings (`cargo clippy -- -D warnings`)
- ✅ Tests must pass (`cargo test`)
- ✅ Docker build must succeed
- ✅ Branch must be up-to-date with `develop`

### PR Review Process

1. **Automated Checks**: CI runs automatically
2. **Code Review**: Maintainers review your code
3. **Feedback**: Address any requested changes
4. **Approval**: Once approved, maintainers will merge

## Release Process

### From `develop` to `main`

Only maintainers create release PRs:

1. **Feature Collection**: Multiple features merged to `develop`
2. **Testing**: Thorough testing on `develop` branch
3. **Release PR**: Maintainer creates PR from `develop` → `main`
4. **Merge**: After approval, merge to `main`
5. **Version and Changelog**: Release Please opens a version/changelog PR on `main`
6. **Review**: Approve workflow runs if prompted. If no CI run exists, close and
   reopen the PR as a maintainer. Wait for all seven jobs on the latest bot update,
   then approve the generated release PR
7. **Release**: Merge that PR; Release Please creates its tag and calls the publisher
8. **Verify**: Check the archive checksum and published container revision

See [Releasing](docs/RELEASING.md) for image tags and repair instructions.

## Development Setup

### Prerequisites

- Rust 1.71 or later
- Docker (for container testing)
- Access to a TrueNAS Scale instance (for integration testing)

### Local Development

```bash
# Clone your fork
git clone https://github.com/mlgruby/truenas-exporter-rs.git
cd truenas-exporter-rs

# Install dependencies
cargo build

# Run tests
cargo test

# Run locally
cp .env.example .env
# Edit .env with your TrueNAS credentials
cargo run
```

### Running CI Checks Locally

Use the Makefile for convenience:

```bash
# Run all development checks (format, lint, test)
make dev

# Individual checks
make fmt          # Format code
make fmt-check    # Check formatting
make clippy       # Run linter
make test         # Run tests
make build        # Build release binary

# Docker
make docker-build
make docker-run
```

Or run cargo commands directly:

```bash
# Format check
cargo fmt --check

# Linting
cargo clippy --all-features -- -D warnings

# Tests
cargo test --all-features

# Build
cargo build --release

# Docker build
docker build -t truenas-exporter:test .
```

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Use meaningful variable and function names
- Add doc comments (`///`) for public APIs
- Add module comments (`//!`) for new modules
- Keep functions focused and small
- Write self-documenting code

## Testing

### Test Structure

- **Unit tests**: In `src/` files using `#[cfg(test)] mod tests`
- **Integration tests**: In `tests/` directory
- **Doc tests**: In documentation comments

### Guidelines

- Add tests for new functions
- Test both success and error cases
- Use descriptive test names
- Ensure all tests pass before submitting PR
- Aim for meaningful coverage (not just 100%)

### Running Tests

```bash
# All tests
cargo test

# Specific test file
cargo test --test metrics_test

# Doc tests only
cargo test --doc

# With output
cargo test -- --nocapture
```

## Documentation

### What to Document

- **Public APIs**: All public functions, structs, and modules
- **Module purpose**: What the module does and why
- **Examples**: Working code examples in doc comments
- **Design decisions**: Why you chose a particular approach

### Documentation Style

```rust
//! Module-level documentation
//!
//! Explains what this module does and provides context.

/// Function documentation
///
/// # Arguments
///
/// * `param` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Example
///
/// ```
/// use truenas_exporter::example;
/// let result = example();
/// ```
pub fn example() -> Result<()> {
    // Implementation
}
```

### Updating Documentation

- Update `README.md` if adding user-facing features
- Update `CHANGELOG.md` (maintainers will finalize)
- Run `cargo doc --open` to preview generated docs
- Include examples for new functionality

## Getting Help

- **Issues**: Check existing issues or create a new one
- **Discussions**: Use GitHub Discussions for questions
- **Documentation**: Check the [README](README.md) and generated docs

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Help others learn and grow
- Follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct)

## Common Tasks

### Adding a New Metric

1. Add the metric to `src/metrics.rs` (definition and registration)
2. Add collection logic to `src/server.rs`
3. Add the API type to `src/truenas/types.rs` (if needed)
4. Add tests to `tests/metrics_test.rs`
5. Update `README.md` metrics list
6. Update `CHANGELOG.md`

### Adding a New API Endpoint

1. Add type definitions to `src/truenas/types.rs`
2. Add query method to `src/truenas/client.rs`
3. Add tests to `tests/types_test.rs`
4. Document the new types with examples

## License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 License.

---

Thank you for contributing to TrueNAS Exporter! 🚀
