# Vega (वेग) — Roadmap
> Zero-Config pure Axum Routing Framework

---

## Vision

Ship a framework where a Rust developer can:
1. Run `vega new my-app`
2. Drop `.rs` files in `pages/` returning String/HTML payloads (e.g. HTMX friendly structures)
3. Run `vega dev`
4. Have a fully working high-performance backend application handling all HTTP REST needs seamlessly.

---

## Release Overview

| Version | Milestone | Status |
|---------|-----------|--------|
| v0.1.0 | Initial prototype hybrid structures | ✅ Completed |
| v0.5.0 | Complete Axum standard router implementations | ✅ Completed |
| v0.9.0 | Pure Zero-Config Codegen architecture complete | ✅ Completed |
| v0.9.5 | HTML Templates and HTMX integration primitives | 🔲 Planned |
| v1.0.0 | Stable, production-ready release | 🔲 Planned |

---

## v0.9.0 — Zero Config Solidification
**Goal:** Prove the file-based zero-config generator scales to production limits safely.

### Core Tasks
- [x] Standardize directory scanning mechanism (`vega-router`)
- [x] Convert dynamic file names `[slug].rs` to Axum `:slug` paths correctly
- [x] Implicit layout/middleware routing mechanisms `middleware.rs`
- [x] Automated root Application State tracking capabilities via `app.rs`
- [x] Robust POST/GET method bindings without manual router chains

---

## v0.9.5 — Component Standardization & Templating
**Goal:** Augment the pure String handling natively to avoid boilerplate string concat mechanisms.

### Core Tasks
- [ ] Internal Template Macro Integration (`vega_html!`)
  - [ ] Implement an ergonomic HTML macro allowing simple variable embeddings directly into Semantic HTML strings seamlessly.
- [ ] HTMX Library Support
  - [ ] Inject base HTMX libraries implicitly inside the generated scaffolds simplifying user onboarding.
  - [ ] Support implicit request handlers recognizing HTMX headers natively using custom Axum extractors.
- [ ] Built-in Server Component Abstractions
  - [ ] Establish standard directory guidelines for `<Component />` generation avoiding complex dependencies.

---

## v1.0.0 — Stable Release
**Goal:** Production-ready, well-documented, purely native backend operations.

### Core Tasks
- [ ] Stabilize API Codegen outputs establishing immutable guarantees
- [ ] Complete Documentation Site purely built with Vega
- [ ] Produce definitive Benchmark suites against raw Axum and pure Next.js variants
- [ ] Formal CLI (`vega-cli`) publish across crates.io

---

## Post v1.0 — Future Ideas

- **Database Adapters** — seamless abstractions allowing simple `pub db: PgPool` integrations across the entire page lifecycle securely.
- **Vega Auth** — Core abstractions handling Session authentication, OAuth, JWT mappings leveraging Axum middlewares perfectly.
- **Edge Delivery** — Establish pathways integrating compiled Vega binaries elegantly into Cloudflare architectures securely.
- **Plugin System** — Establish pipeline hooks integrating external frameworks securely extending internal compilation processes easily.

---

## Non-Goals (Things Vega Will Never Do)

- Re-introduce complex WebAssembly DOM Hydration systems natively. (Vega trusts pure HTML returns).
- Abstract away Rust's ownership models through unsafe abstractions.
- Replace Axum capabilities (Vega relies entirely on Tower/Axum primitives).
