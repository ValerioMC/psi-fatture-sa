# PSI Fatture — project rules

Desktop app (Tauri 2 + Rust backend, Vue 3 + TypeScript frontend, SQLite) for invoicing by
Italian psychologists. User-facing text is Italian; code, identifiers and comments are English.

Layering, one-type-per-file, typed errors and test placement are described in
`README.md` → *Convenzioni del backend*. These rules extend them; read both before changing code.

## Code quality

- **Readable first.** A function reads top to bottom as the steps it performs. If a block needs a
  comment to say *what* it does, extract it into a private function whose name says it.
- **Small functions.** Aim under ~30 lines. A long function is a list of steps: each step that
  does one recognisable thing (a rule, a lookup, a mapping) becomes a private function, even if it
  is used once. Do not extract one-liners: a function must carry real logic or a real name.
- **Names over comments.** Comments explain *why* or a rule's edge, never restate the name.
  Max 30 words on a function, 50 on a type/module. No `TODO` in code.
- **No duplication.** Two blocks building the same record, grouping the same data or formatting
  the same value share one function. Check before adding a near-copy.
- **No useless abstraction.** No trait with one implementation unless it is an external boundary
  (HTTP, SMTP, file system, clock, secrets) that tests must replace. No generic "helpers" module.
- **Composition over inheritance**, and **dependency inversion at the boundaries**: services
  receive gateways as `Arc<dyn Trait>` from `AppState`; they never build an HTTP/SMTP client.
- **Errors are explicit.** Return `Result<_, AppError>`. Never `unwrap_or_default()` a parse of
  stored data to hide a failure; `expect` only at startup in `lib.rs`.

## Backend structure

- **One service per entity.** `service/<entity>_service.rs`, or a package `service/<entity>/`
  when the entity needs collaborators. In a package only `<entity>_service` is `pub`; the other
  files (rules, value types) are private modules with `pub(super)` items, re-exported by `mod.rs`.
  Example: `service/invoice/` (`invoice_service`, `invoice_guard`, `ClientSessions`, `InvoiceContent`).
- **Controllers forward.** A Tauri command validates nothing and maps nothing beyond calling one
  service function with `&state.db` and the gateways it needs.
- **Repositories return data, not presentation.** Numbers as numbers, dates as stored; formatting
  (`"007"`, `05/03`, `€ 70,00`) happens in the service or the PDF/email layer.
- **Rules live in one place.** A business rule that refuses a change (numbering, Sistema TS
  locks) is a named `ensure_*` function returning `AppError::Conflict` or `Invalid`.

## Frontend structure

- Every backend call goes through `src/api.ts`; views and components never call `invoke` directly.
- Domain logic lives in `src/utils/*.ts` (pure, unit-tested) or a composable; a `.vue` file keeps
  only presentation state. Totals shown to the user come from the backend (`preview_invoice_totals`).
- `<script setup lang="ts">`, typed `defineProps`/`defineEmits`, no `any`.

## Before handing work back

```bash
make fmt lint test     # cargo fmt --check, clippy -D warnings, vue-tsc, vitest, cargo test
```

Update `README.md` when structure, commands or configuration change.
