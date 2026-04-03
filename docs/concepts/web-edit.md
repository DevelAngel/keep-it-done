# Web UI Edit -- UX Concept

## Abstract

The web UI is a read-only task list with multiple filtered views (My Day, What I Finished, Quick Wins, Recent Changes) by default.
An edit mode exists for emergency edits when CLI access is unavailable.
The CLI remains the primary mutation path.

## Context

- Primary mutation: `kid-cli` → tarpc/TCP → `kid-server` RPC handler → cache → file
- Web mutation: HTTP → Leptos server function (in-process, `kid-server`) → cache → file
- Same business logic layer; no divergence in validation or storage
- Target: mobile-first browser, family members, rare use

## Data Model (Current)

**Set on create:** `id`, `summary`, `status`

**Editable via web UI:**

- `summary` — free text
- `priority` — A / B / C
- `due_date` — free-text guess or precise date
- `start_date` — free-text guess or precise date
- `time_estimate` — free-text guess or precise duration
- `context` — free text
- `notes` — free text

**Managed by the server:** `status` carries the timestamp of the last status change. `edited_at` does not exist yet — see conflict handling.

## Edit Mode

A toggle in the app bar activates edit mode globally for the session. The toggle must be visually prominent — a subtle indicator is insufficient given the behavioral change it triggers.

Read-only mode hides empty fields. Edit mode reveals all fields as inputs.

The completed flow (marking a task done via the checkbox) is unaffected by edit mode state.

## Interaction Model

### Entry point

Tap on a task row opens the detail expansion — identical behavior in both read-only and edit mode. No new tap semantics.

### Summary field

`summary` is the first field in the detail expansion. In edit mode it renders as a text input. Editing the summary costs one extra tap compared to a direct inline edit, which is acceptable given its rarity.

### Structured fields (in detail expansion)

- `priority` — 3-button toggle: A / B / C
- `due_date` — 2-button toggle: Guess / Precise + free text or native date picker
- `start_date` — same as `due_date`
- `time_estimate` — preset buttons (15 min / 30 min / 1h / 2h / half day) + free text fallback
- `context` — text input
- `notes` — autogrow textarea

`status` is not editable in the detail expansion. The only status transition available via the web UI is Done ↔ ToDo through the existing checkbox flow.

No field requires an explicit save button. Each change is submitted on blur or on toggle tap.

All details mutations go through the existing `update` RPC method via a Leptos server function. The `rename` RPC method handles `summary` separately.

## Write Path and Conflict Handling

> **Prerequisite:** an `edited_at` field must be added to task details before conflict detection can be implemented. The field is set by `kid-server` on every details write.

Once `edited_at` exists, the client sends its locally cached value alongside each write. The server rejects the write if the stored `edited_at` is newer (another writer mutated the task concurrently).

On rejection: optimistic UI update rolls back to the previous value. An inline message appears within the detail expansion: "Updated by another client — reload to see current state." No modal, no navigation loss.

Offline writes are not queued. On a local family network, connectivity loss is rare; silent failure on network error is acceptable.

## Out of Scope

- Task creation via web UI (CLI only)
- Batch editing
- Dependency editing
- Offline support
- Floating action button (FAB) for edit mode toggle — FABs obscure content and conflict with scrolling on mobile; rejected in favour of a header icon + active-state banner.
