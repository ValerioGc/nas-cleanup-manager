# Development of nas-cleanup-manager

The Vue/TypeScript UI is in `src/`; the Rust/Axum backend is in `backend/axum/`. The shared scripts are in the library `@valeriogc/nas-base-app`; manifest and version are in `nas-app.json`, `package.json` and the Cargo metadata. `src/assets/logo.svg` is the source of the logo and the UI icons are in `src/assets/icons/`.

## Conventions

Use the Composition API with `<script setup lang="ts">`, imports through the `@/` alias, scoped SFC styles, CSS names with `_` and UI texts in the catalogues `src/i18n/locales/en.ts` and `it.ts`. Code, method names, test descriptions, this document and the README are in English. The NAS theme comes from the context of the iframe (`useNasLauncherContext` of the library, bound to the catalogue by `src/composables/useNasLauncherContext.ts`); the app asks the shell for nothing else. Layout and sidebar are `NasAppLayout`, the dialogs `CommonModal` on `NasModal`.

The FE tests are in `src/tests/unit`, `component`, `scripts`, `e2e`, with sub-paths mirroring `src/`. The Rust tests are in `backend/axum/tests/unit`, with a shared support (`support.rs`: in-memory database and a fake launcher on a socket that answers like the real one and keeps the requests). The E2E tests start the real backend on a temporary database, next to a fake launcher (`src/tests/e2e/fixtures/launcher.mjs`).

## How it cleans

In the backend, `catalog.rs` holds the items: those of the system, with their group and warning, are the identifiers of the fixed list of the launcher; every cache declared by an app is `cache:<app>:<id>`. `cleaner.rs` measures and cleans one item after the other: the system items with `POST /api/apps/self/cleanup` on the socket of the launcher (`launcher.rs`), which runs them as root in `nas-os-cleanup.service`, the caches with `GET`/`DELETE /api/caches`. One cleanup at a time, shown among the operations of the launcher, written in the history and, when the settings say so, closed by a notification. `store.rs` holds settings, choice, scheduled cleanups and history (`migrations/0001_cleanup.sql`); `schedule.rs` and `scheduler.rs` are those of nas-backup-manager, with `systemd-analyze calendar` for the next run. The app tells the launcher it works in the background only while a scheduled cleanup is on.

## Dependencies and shared library

Dependencies, lockfiles and the use of `@valeriogc/nas-base-app` follow the section "For the
applications" of `DEVELOPMENT.md` in nas-base-app. In short, run `pnpm run lock:npm` after
`pnpm add`, `update` or `remove`, and keep a token with `read:packages` in `~/.npmrc` to install
the library.

## Checks

```bash
pnpm install
pnpm run format:check
pnpm run lint
pnpm run type-check
pnpm test
pnpm run test:e2e
pnpm run test:coverage
cargo fmt --manifest-path backend/Cargo.toml --all --check
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path backend/Cargo.toml
pnpm run nas:package
pnpm run nas:test
```
