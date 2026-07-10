# Pythia Frontend — Test Profile Configurator (Design)

**Date:** 2026-07-10
**Status:** Approved design, ready for implementation planning

## Summary

An Angular web app for building **test profiles** for Pythia's HIL (hardware-in-the-loop)
test mode. A test profile is a named collection of scheduled CAN frames. The app lets a
user select or create a test profile, then view, add, and delete the CAN frames that belong
to it.

Analog and GPIO outputs (shown in the source mockup) are **out of scope** for this effort;
their tabs appear in the UI but are disabled ("coming soon").

The app is branded **Pythia** (the source mockup was branded "Calypso").

## Scope

**In scope**
- Angular frontend only (`pythia-frontend/`).
- Select an existing test profile from a dropdown.
- Create a new test profile.
- List the CAN frames of the selected profile.
- Add a CAN frame to the selected profile.
- Delete a CAN frame; delete a profile.
- Light beige/orange Angular Material theme, Pythia branding.

**Out of scope (this effort)**
- Analog Out / GPIO Out configuration (tabs present but disabled).
- Unit/integration tests (deliberately deferred to a separate effort).
- nginx / Docker deployment concerns.
- Rust backend implementation. The GET/POST endpoints already exist. The two DELETE
  endpoints are assumed to be added by the backend owner; this doc specifies their contract.

## Backend contract

Base URL in dev: `http://localhost:3000` (reached via the Angular dev-server proxy so the
app can call same-origin paths `/profiles` and `/messages`).

### Data shapes

A CAN message as returned by the backend (`TestCanMessageEntry`):

```json
{
  "id": 12,
  "profile_id": 3,
  "can_id": 513,
  "is_extended": 0,
  "data": [1, 0, 0, 0, 0, 0, 0, 0],
  "mode": "oneshot",
  "offset_ms": 500,
  "period_ms": null
}
```

- `can_id` — integer. Standard frames: `0..=0x7FF`. Extended frames: `0..=0x1FFFFFFF`.
- `is_extended` — `0` (standard / STD) or `1` (extended / EXT).
- `data` — **JSON array of byte integers** (0–255), length 0–8. **Not** a hex string.
  On the frontend the payload is entered as a single **number** and converted to this byte
  array at the API boundary (see `payload.util.ts`).
- `mode` — `"oneshot"` or `"broadcast"`.
- `offset_ms` — integer ≥ 0 (the mockup's "T-START").
- `period_ms` — integer > 0 for `broadcast`; `null` for `oneshot`. Broadcast **requires** a
  period; oneshot **must not** have one.

A test profile (`TestProfile`): `{ "id": 3, "name": "PROFILE_001" }`.

### Endpoints

| Method & path | Body / query | Success | Errors |
|---|---|---|---|
| `GET /profiles` | — | `200` `string[]` (names, alphabetical) | `500` |
| `POST /profiles` | `{ "name": string }` | `201` `TestProfile` | `409` duplicate name, `500` |
| `GET /messages?profile=<name>` | — | `200` `TestCanMessageEntry[]` (asc `offset_ms`) | `404` profile not found, `500` |
| `POST /messages?profile=<name>` | `NewCanMessageInput` | `201` `TestCanMessageEntry` | `400` invalid frame (DB constraint), `404` profile not found, `500` |

`NewCanMessageInput` (POST body): `can_id`, `is_extended`, `data` (byte array), `mode`,
`offset_ms`, `period_ms` (nullable). No `id`/`profile_id` (the latter is resolved from the
`profile` query param).

### DELETE endpoints (to be added by backend owner — contract specified here)

| Method & path | Success | Errors |
|---|---|---|
| `DELETE /messages?id=<messageId>` | `204 No Content` | `404` if the message id does not exist |
| `DELETE /profiles?name=<name>` | `204 No Content` | `404` if the profile does not exist |

Deleting a profile cascade-deletes its messages (the DB already declares
`ON DELETE CASCADE`).

## Architecture (Approach A: signal-based facade + service-per-resource)

Standalone Angular components with signals. A store facade holds state; components read
signals and call store methods. The API services are the only layer aware of the wire
format, so hex↔byte conversion and error mapping are isolated there.

### Layers

- **`models/`** — TypeScript types mirroring the contract:
  - `TestProfile` `{ id: number; name: string }`
  - `CanMessage` — domain view of a frame (see note on `data` below)
  - `NewCanMessageInput` — POST body shape
  - `CanMode = 'oneshot' | 'broadcast'`
  - `CanFormat = 'std' | 'ext'` (maps to `is_extended` 0/1)

  Note: components/store work with `data` as a `number[]` (byte array). The single-number
  payload entry and display is a presentation concern handled via the payload utility at the
  edges (form input + table cell), not stored on the domain model.

- **`payload.util.ts`** — pure functions for the single-number payload. Uses `BigInt`
  internally because 8 bytes (up to 2^64−1) exceeds JS's safe integer range:
  - `numberToBytes(value: bigint): number[]` — big-endian byte array, minimal length,
    ≤ 8 bytes. Rejects negatives and values ≥ 2^64.
  - `bytesToNumber(bytes: number[]): bigint` — inverse; `[1,0]` → `256n`.
  - `parsePayload(input: string): bigint` — parse a decimal string to `bigint`.
  - `parseCanId(input: string): number` / `formatCanId(id: number): string` — hex CAN id
    with `0x` prefix handling.

- **`api/`**
  - `ProfileApiService` — `listProfiles()`, `createProfile(name)`, `deleteProfile(name)`.
  - `MessageApiService` — `listMessages(profile)`, `createMessage(profile, input)`,
    `deleteMessage(id)`. Passes the domain `number[]` data straight through as the wire
    array, and maps HTTP error codes to typed app errors. (Number↔byte-array conversion
    lives in the form/table via `payload.util`.)

- **`state/profile.store.ts`** — injectable store exposing signals:
  - `profiles: Signal<string[]>`
  - `selectedProfile: Signal<string | null>`
  - `frames: Signal<CanMessage[]>`
  - `loading: Signal<boolean>`, `error: Signal<string | null>`
  - Methods: `loadProfiles()`, `selectProfile(name)`, `createProfile(name)`,
    `addFrame(input)`, `deleteFrame(id)`, `deleteProfile(name)`.
  - Orchestration: mutations call the API, then re-fetch the affected data (frames list or
    profile list). No optimistic caching — always consistent with the backend, simplest to
    reason about at this scale.

### Components

- **`ShellComponent`** — Pythia-branded header (replaces "CALYPSO"), a mode/status strip,
  and a footer. Hosts a tab bar: **CAN Frames** (active) with **Analog Out** and
  **GPIO Out** present but disabled ("coming soon").
- **`ProfileSelectorComponent`** — Material `mat-select` bound to `profiles`, plus a
  "New profile…" action that opens a small dialog (name input → `createProfile`, surfaces
  `409` duplicate as a field error).
- **`CanFrameTableComponent`** — Material table over `frames`. Columns: CAN ID (hex),
  Format chip (STD/EXT), Content (decimal number via `bytesToNumber`), Mode chip
  (BCAST/1-SHOT), T-Start (ms), Period (ms or "—"), and a delete button per row →
  `deleteFrame(id)` (with confirm).
- **`AddFrameFormComponent`** — Material reactive form:
  - CAN ID (hex text), Format select (Standard/Extended), Content (single decimal number),
    Mode select (Single Shot/Broadcast), T-Start (ms).
  - **Period (ms) field is shown only when Mode = Broadcast** (and required there).
  - Client-side validation mirrors DB constraints: CAN id within range for the chosen
    format, payload number ≥ 0 and fits in 8 bytes (< 2^64, so `numberToBytes` yields
    ≤ 8 bytes), `offset_ms ≥ 0`, `period_ms > 0` required iff broadcast. On submit the
    payload number is converted to a byte array via `numberToBytes` → `addFrame(input)`;
    backend `400`/`404` surfaced as errors.

### Data flow

1. On load, `loadProfiles()` populates the dropdown.
2. Selecting a profile calls `selectProfile(name)` → `GET /messages?profile=<name>` →
   `frames` signal updates → table renders.
3. Add frame → `POST /messages` → re-fetch frames for the selected profile.
4. Delete frame → `DELETE /messages?id=` → re-fetch frames.
5. Create/delete profile → `POST`/`DELETE /profiles` → re-fetch profile list (and clear or
   reselect selection as appropriate).

The payload number↔byte-array conversion happens in the form (on submit) and table (on
render) via `payload.util`; the store and API layer operate on the `number[]` byte array,
matching the wire format directly.

### Error handling

Backend errors surface as Material snackbars (and inline field errors where relevant):
- `409` — profile name already exists (inline on the create-profile dialog).
- `400` — invalid frame; show the constraint message from the backend.
- `404` — profile/message not found; refresh state.
- Network/`500` — generic "Something went wrong" with a retry affordance.

## Visual design

- Light theme: beige/cream surfaces with orange accent (a lighter reinterpretation of the
  dark instrument mockup). Angular Material custom theme.
- Pythia branding in the header in place of Calypso.
- Retain the mockup's information layout: header + status, tabbed sections, a frames table,
  and an "Add Frame" form beneath it.

## Testing

Deliberately deferred. No unit/integration tests in this effort; a separate testing effort
will cover `payload.util`, the store, and component validation. Code should be structured to
keep those units testable (pure payload utility, API-mockable store).

## Assumptions & decisions

- DELETE endpoints use query params (`?id=`, `?name=`) and return `204`, matching the
  existing query-param API style. Backend owner will implement them.
- The CAN frame payload is entered on the frontend as a single **decimal** number (up to
  8 bytes, i.e. < 2^64, handled via `BigInt`) and converted to the backend's byte array at
  the form boundary. CAN ID remains hex. The backend continues to store `data` as a byte
  array.
- Add-frame persists immediately (one `POST` per frame); no client-side batching/staging.
- Mutations re-fetch rather than optimistically mutate local state.
- Analog/GPIO tabs are shown-but-disabled for fidelity rather than removed.
