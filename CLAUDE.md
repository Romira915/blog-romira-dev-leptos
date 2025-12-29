# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Personal blog built with Leptos (Rust web framework) using SSR + client-side hydration. Aggregates content from multiple sources: Newt CMS (primary), WordPress (PR Times), and Qiita.

## Build Commands

```bash
# Install dependencies (requires just command runner)
brew install just
just setup

# Development server with hot reload
just watch    # Runs stylance --watch and cargo leptos watch

# Linting and testing (CI checks)
cargo fmt --all -- --check
leptosfmt --check ./**/*.rs
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features

# Production build (aarch64 target)
cargo leptos build --release --bin-cargo-args="--target=aarch64-unknown-linux-gnu"
```

## Architecture

### Workspace Structure

- **app/** - Isomorphic library with conditional compilation
  - `#[cfg(feature = "ssr")]` for server-side only code
  - `#[cfg(feature = "hydrate")]` for client-side only code
  - `front/` - UI components and pages
  - `server/` - SSR code: config, services, models, HTTP utilities
  - `common/` - Shared handlers and DTOs
- **front/** - WASM entry point (`hydrate()` function)
- **server/** - Axum binary that serves SSR content

### Key Patterns

**Server Functions**: RPC handlers using `#[server]` macro
```rust
#[server(input = GetUrl, endpoint = "get_articles_handler")]
pub async fn get_articles_handler() -> Result<Vec<ArticleDto>, ServerFnError>
```

**Context-Based DI**: Services injected via `AppState` and accessed with `expect_context::<AppState>()`

**Stylance CSS Modules**: Component styles in `.module.scss` files, compiled to `style/stylance/_index.scss`

**SSR Routing**: Routes use `ssr=SsrMode::Async` for async server-side rendering

### Feature Flags

`ssr` and `hydrate` features are **mutually exclusive** - never enable both together. This is enforced in `cargo-all-features` config.

## Environment Variables

Copy `.env.example` to `.env` for development. Required variables:
- `NEWT_CDN_API_TOKEN` / `NEWT_API_TOKEN` - Newt CMS API tokens
- `QIITA_API_TOKEN` - Qiita API token
- `NEW_RELIC_LICENSE_KEY` - Observability (optional for dev)
- `HOST_NAME` - Server hostname

## Toolchain

Uses **Rust nightly** (specified in `rust-toolchain.toml`). Key tools:
- `cargo-leptos` v0.2.27 - Build orchestration
- `leptosfmt` v0.1.32 - Leptos component formatter
- `stylance-cli` v0.5.4 - CSS module compiler
- `wasm-bindgen-cli` v0.2.100 - WASM bindings

## Value Object Guidelines

- **Input (Handler → Service)**: Use Value Objects to enforce validation
- **Output (Service → DTO)**: Use plain `String` (trust data retrieved from DB)

```rust
// Input: enforce validation (for publishing)
pub fn save(title: PublishedArticleTitle, slug: PublishedArticleSlug, body: &str) -> Result<...>

// Output: return plain types
pub fn fetch(id: Uuid) -> Result<ArticleDto>  // ArticleDto.title is String
```

## Development Notes

- **Always use `commit-session` Skill for commits** - do not manually git add/commit
- **Run `cargo sqlx prepare` when modifying SQL queries** - execute `cargo sqlx prepare --workspace -- --all-targets` before committing when adding/changing queries used by `sqlx::query!` macro. Include generated JSON files in `.sqlx/` directory in commits
- **Do NOT use `mod.rs`** - Use modern Rust module style (`foo.rs` + `foo/` directory) instead of `foo/mod.rs`
- Dev server runs at http://127.0.0.1:3000 with hot reload on port 3001
- WASM release profile uses aggressive optimizations (LTO, opt-level='z')
- Images are optimized via imgix CDN (URL transformation in `server/utils/url.rs`)
- Date format is Japanese: "YYYY年MM月DD日" (JST timezone)