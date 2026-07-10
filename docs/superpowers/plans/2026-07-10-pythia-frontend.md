# Pythia Frontend — Test Profile Configurator Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an Angular app (`pythia-frontend/`) that lets a user select or create a test profile and view/add/delete its CAN frames, talking to the existing Pythia Rust backend.

**Architecture:** Standalone Angular components with signals (Approach A from the spec). A `ProfileStore` facade holds signal state; `ProfileApiService`/`MessageApiService` wrap `HttpClient` and own the wire contract; components read signals and call store methods. Angular Material provides UI, themed light beige/orange.

**Tech Stack:** Angular 20 (standalone + signals), Angular Material 20, TypeScript, SCSS, RxJS (via HttpClient), Node 22 LTS. No test framework in this effort (tests deferred per spec).

## Global Constraints

- App lives in `pythia-frontend/` at the repo root (sibling of `pythia-backend/`).
- Node 22 LTS (Angular 20 requires Node `^20.19 || ^22.12 || ^24`).
- Branding is **Pythia** (never "Calypso").
- Backend base URL in dev is `http://localhost:3000`, reached via the Angular dev-server proxy so the app calls same-origin `/profiles` and `/messages`.
- CAN frame `data` on the wire is a JSON **array of byte integers** (0–255), length 0–8. The frontend enters the payload as a single **decimal** number and converts via `payload.util` (`BigInt`-based, full 8-byte range).
- `mode` values are the strings `"oneshot"` and `"broadcast"`. `is_extended` is `0` (STD) or `1` (EXT).
- CAN id range: STD `0..=0x7FF`, EXT `0..=0x1FFFFFFF`. `offset_ms >= 0`. `period_ms > 0` required iff broadcast, must be null for oneshot.
- **No unit tests in this effort.** Each task is verified by a successful production build (`npm run build`, which typechecks) and, where noted, a manual visual check via `npm start`. Commit after each task.
- DELETE endpoints (`DELETE /messages?id=`, `DELETE /profiles?name=`, both `204`) are assumed to exist on the backend; the frontend calls them.

---

## File Structure

```
pythia-frontend/
  proxy.conf.json                          # dev proxy: /profiles,/messages -> :3000
  src/
    styles.scss                            # global styles + Material theme (beige/orange)
    index.html                             # title "Pythia"
    main.ts                                 # bootstrap (generated, add providers)
    app/
      app.config.ts                        # providers: HttpClient, animations
      app.ts                               # root component -> hosts ShellComponent
      models/
        can.models.ts                      # TestProfile, CanMessage, NewCanMessageInput, CanMode, CanFormat
      utils/
        payload.util.ts                    # numberToBytes/bytesToNumber/parsePayload/parseCanId/formatCanId
      api/
        profile-api.service.ts             # listProfiles/createProfile/deleteProfile
        message-api.service.ts             # listMessages/createMessage/deleteMessage
      state/
        profile.store.ts                   # signal store facade
      components/
        shell/shell.ts, shell.html, shell.scss
        profile-selector/profile-selector.ts (+ .html/.scss)
        new-profile-dialog/new-profile-dialog.ts (+ .html/.scss)
        can-frame-table/can-frame-table.ts (+ .html/.scss)
        add-frame-form/add-frame-form.ts (+ .html/.scss)
```

---

## Task 0: Environment setup + scaffold Angular app

**Files:**
- Create: `pythia-frontend/` (Angular CLI output)
- Create: `pythia-frontend/proxy.conf.json`
- Modify: `pythia-frontend/package.json` (start script uses proxy)
- Modify: `pythia-frontend/src/index.html` (title)
- Modify: repo `.gitignore` (ignore `pythia-frontend/node_modules`, `.angular`, `dist`)

**Interfaces:**
- Produces: a buildable Angular 20 workspace with Angular Material installed and a working dev proxy. Later tasks add files under `src/app/`.

- [ ] **Step 1: Ensure Node 22 is available**

Node is not installed on this machine. Install via nvm (no sudo, installs into `$HOME`):

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
export NVM_DIR="$HOME/.nvm"; [ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"
nvm install 22
node --version   # expect v22.x
npm --version
```

(If a different Node install method is preferred, any Node satisfying `^20.19 || ^22.12 || ^24` works. Re-source nvm in each new shell: `export NVM_DIR="$HOME/.nvm"; . "$NVM_DIR/nvm.sh"`.)

- [ ] **Step 2: Scaffold the workspace**

Run from the repo root (`/home/caiod/NER/Pythia`):

```bash
npx -y @angular/cli@20 new pythia-frontend \
  --style=scss --routing=false --ssr=false --skip-git --skip-tests --defaults
```

Expected: a `pythia-frontend/` directory is created and dependencies install. `--skip-tests` omits `.spec.ts` files (tests are deferred).

- [ ] **Step 3: Add Angular Material**

```bash
cd pythia-frontend
npx ng add @angular/material@20 --skip-confirmation --theme=custom --typography --animations=enabled
```

Expected: Material + CDK added to `package.json`, `provideAnimations()` (or equivalent) wired into `app.config.ts`, and `styles.scss` set up for a custom theme. If the schematic adds a prebuilt theme import to `styles.scss`, remove it — Task 4 defines the custom beige/orange theme.

- [ ] **Step 4: Create the dev proxy**

Create `pythia-frontend/proxy.conf.json`:

```json
{
  "/profiles": { "target": "http://localhost:3000", "secure": false, "changeOrigin": true },
  "/messages": { "target": "http://localhost:3000", "secure": false, "changeOrigin": true }
}
```

- [ ] **Step 5: Wire the proxy into the start script**

In `pythia-frontend/package.json`, set the `start` script to use the proxy:

```json
"start": "ng serve --proxy-config proxy.conf.json",
```

- [ ] **Step 6: Set the page title**

In `pythia-frontend/src/index.html`, set `<title>Pythia</title>`.

- [ ] **Step 7: Ignore build artifacts in git**

Append to the repo-root `.gitignore`:

```
# Angular frontend
pythia-frontend/node_modules/
pythia-frontend/.angular/
pythia-frontend/dist/
```

- [ ] **Step 8: Verify the build**

```bash
cd pythia-frontend && npm run build
```

Expected: build completes with no errors (this is the typecheck gate used throughout).

- [ ] **Step 9: Commit**

```bash
cd /home/caiod/NER/Pythia
git add pythia-frontend .gitignore
git commit -m "chore: scaffold pythia-frontend Angular app with Material + dev proxy"
```

---

## Task 1: Domain models + payload utility

**Files:**
- Create: `pythia-frontend/src/app/models/can.models.ts`
- Create: `pythia-frontend/src/app/utils/payload.util.ts`

**Interfaces:**
- Produces:
  - `CanMode = 'oneshot' | 'broadcast'`
  - `CanFormat = 'std' | 'ext'`
  - `interface TestProfile { id: number; name: string }`
  - `interface CanMessage { id: number; profile_id: number; can_id: number; is_extended: number; data: number[]; mode: CanMode; offset_ms: number; period_ms: number | null }`
  - `interface NewCanMessageInput { can_id: number; is_extended: number; data: number[]; mode: CanMode; offset_ms: number; period_ms: number | null }`
  - `numberToBytes(value: bigint): number[]`
  - `bytesToNumber(bytes: number[]): bigint`
  - `parsePayload(input: string): bigint`
  - `parseCanId(input: string): number`
  - `formatCanId(id: number): string`
  - `MAX_PAYLOAD: bigint` (`2n ** 64n - 1n`)

- [ ] **Step 1: Create the models**

Create `pythia-frontend/src/app/models/can.models.ts`:

```typescript
/** Transmission mode of a CAN frame, matching the backend's `mode` column. */
export type CanMode = 'oneshot' | 'broadcast';

/** Frame format. Maps to the backend's integer `is_extended` (std=0, ext=1). */
export type CanFormat = 'std' | 'ext';

/** A named collection of CAN frames. */
export interface TestProfile {
  id: number;
  name: string;
}

/** A CAN frame as returned by the backend (`GET /messages`). */
export interface CanMessage {
  id: number;
  profile_id: number;
  can_id: number;
  is_extended: number; // 0 | 1
  data: number[]; // 0-8 bytes, each 0-255
  mode: CanMode;
  offset_ms: number;
  period_ms: number | null;
}

/** Body for `POST /messages` — no id (DB-assigned) or profile_id (from query param). */
export interface NewCanMessageInput {
  can_id: number;
  is_extended: number; // 0 | 1
  data: number[];
  mode: CanMode;
  offset_ms: number;
  period_ms: number | null;
}
```

- [ ] **Step 2: Create the payload utility**

Create `pythia-frontend/src/app/utils/payload.util.ts`:

```typescript
/** Largest payload value that fits in 8 bytes. */
export const MAX_PAYLOAD = 2n ** 64n - 1n;

/** Largest CAN id for a standard (11-bit) frame. */
export const MAX_STD_ID = 0x7ff;
/** Largest CAN id for an extended (29-bit) frame. */
export const MAX_EXT_ID = 0x1fffffff;

/**
 * Convert a non-negative payload value into a big-endian byte array of minimal
 * length (0-8 bytes). `0n` yields `[]`. Throws if negative or > 2^64-1.
 */
export function numberToBytes(value: bigint): number[] {
  if (value < 0n) throw new Error('payload must be non-negative');
  if (value > MAX_PAYLOAD) throw new Error('payload exceeds 8 bytes');
  const bytes: number[] = [];
  let v = value;
  while (v > 0n) {
    bytes.unshift(Number(v & 0xffn));
    v >>= 8n;
  }
  return bytes;
}

/** Inverse of {@link numberToBytes}: big-endian byte array to a bigint. */
export function bytesToNumber(bytes: number[]): bigint {
  return bytes.reduce((acc, b) => (acc << 8n) | BigInt(b & 0xff), 0n);
}

/**
 * Parse a user-entered decimal string into a payload bigint.
 * Throws on empty/non-numeric input.
 */
export function parsePayload(input: string): bigint {
  const trimmed = input.trim();
  if (!/^\d+$/.test(trimmed)) throw new Error('payload must be a whole number');
  return BigInt(trimmed);
}

/**
 * Parse a user-entered CAN id (hex, with or without `0x` prefix) into a number.
 * Throws on non-hex input.
 */
export function parseCanId(input: string): number {
  const trimmed = input.trim().replace(/^0x/i, '');
  if (!/^[0-9a-f]+$/i.test(trimmed)) throw new Error('CAN id must be hex');
  return parseInt(trimmed, 16);
}

/** Format a CAN id as an uppercase hex string with `0x` prefix. */
export function formatCanId(id: number): string {
  return '0x' + id.toString(16).toUpperCase();
}
```

- [ ] **Step 3: Verify the build**

```bash
cd pythia-frontend && npm run build
```

Expected: build passes (files compile; unused-so-far exports are fine).

- [ ] **Step 4: Commit**

```bash
cd /home/caiod/NER/Pythia
git add pythia-frontend/src/app/models pythia-frontend/src/app/utils
git commit -m "feat(frontend): add CAN domain models and payload utility"
```

---

## Task 2: API services

**Files:**
- Create: `pythia-frontend/src/app/api/profile-api.service.ts`
- Create: `pythia-frontend/src/app/api/message-api.service.ts`
- Verify: `pythia-frontend/src/app/app.config.ts` provides `HttpClient`

**Interfaces:**
- Consumes: models from Task 1.
- Produces:
  - `ProfileApiService.listProfiles(): Observable<string[]>`
  - `ProfileApiService.createProfile(name: string): Observable<TestProfile>`
  - `ProfileApiService.deleteProfile(name: string): Observable<void>`
  - `MessageApiService.listMessages(profile: string): Observable<CanMessage[]>`
  - `MessageApiService.createMessage(profile: string, input: NewCanMessageInput): Observable<CanMessage>`
  - `MessageApiService.deleteMessage(id: number): Observable<void>`

- [ ] **Step 1: Ensure HttpClient is provided**

Confirm `pythia-frontend/src/app/app.config.ts` includes `provideHttpClient()` in its `providers` array. If missing, add it and the import:

```typescript
import { provideHttpClient } from '@angular/common/http';
// ...within providers: [ ..., provideHttpClient() ]
```

- [ ] **Step 2: Create the profile API service**

Create `pythia-frontend/src/app/api/profile-api.service.ts`:

```typescript
import { HttpClient, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';
import { TestProfile } from '../models/can.models';

/** Thin wrapper over the backend's `/profiles` endpoints. */
@Injectable({ providedIn: 'root' })
export class ProfileApiService {
  private readonly http = inject(HttpClient);

  listProfiles(): Observable<string[]> {
    return this.http.get<string[]>('/profiles');
  }

  createProfile(name: string): Observable<TestProfile> {
    return this.http.post<TestProfile>('/profiles', { name });
  }

  deleteProfile(name: string): Observable<void> {
    return this.http.delete<void>('/profiles', {
      params: new HttpParams().set('name', name),
    });
  }
}
```

- [ ] **Step 3: Create the message API service**

Create `pythia-frontend/src/app/api/message-api.service.ts`:

```typescript
import { HttpClient, HttpParams } from '@angular/common/http';
import { Injectable, inject } from '@angular/core';
import { Observable } from 'rxjs';
import { CanMessage, NewCanMessageInput } from '../models/can.models';

/** Thin wrapper over the backend's `/messages` endpoints. */
@Injectable({ providedIn: 'root' })
export class MessageApiService {
  private readonly http = inject(HttpClient);

  listMessages(profile: string): Observable<CanMessage[]> {
    return this.http.get<CanMessage[]>('/messages', {
      params: new HttpParams().set('profile', profile),
    });
  }

  createMessage(profile: string, input: NewCanMessageInput): Observable<CanMessage> {
    return this.http.post<CanMessage>('/messages', input, {
      params: new HttpParams().set('profile', profile),
    });
  }

  deleteMessage(id: number): Observable<void> {
    return this.http.delete<void>('/messages', {
      params: new HttpParams().set('id', String(id)),
    });
  }
}
```

- [ ] **Step 4: Verify the build**

```bash
cd pythia-frontend && npm run build
```

Expected: build passes.

- [ ] **Step 5: Commit**

```bash
cd /home/caiod/NER/Pythia
git add pythia-frontend/src/app/api pythia-frontend/src/app/app.config.ts
git commit -m "feat(frontend): add profile and message API services"
```

---

## Task 3: ProfileStore (signal facade)

**Files:**
- Create: `pythia-frontend/src/app/state/profile.store.ts`

**Interfaces:**
- Consumes: `ProfileApiService`, `MessageApiService`, models from Tasks 1–2.
- Produces `ProfileStore` (injectable, `providedIn: 'root'`) with readonly signals and methods:
  - Signals: `profiles: Signal<string[]>`, `selectedProfile: Signal<string | null>`, `frames: Signal<CanMessage[]>`, `loading: Signal<boolean>`, `error: Signal<string | null>`
  - Methods: `loadProfiles(): void`, `selectProfile(name: string): void`, `createProfile(name: string): Observable<TestProfile>`, `addFrame(input: NewCanMessageInput): Observable<CanMessage>`, `deleteFrame(id: number): void`, `deleteProfile(name: string): void`, `clearError(): void`

- [ ] **Step 1: Create the store**

Create `pythia-frontend/src/app/state/profile.store.ts`:

```typescript
import { Injectable, inject, signal } from '@angular/core';
import { Observable, tap } from 'rxjs';
import { MessageApiService } from '../api/message-api.service';
import { ProfileApiService } from '../api/profile-api.service';
import { CanMessage, NewCanMessageInput, TestProfile } from '../models/can.models';

/**
 * Signal-based facade over the profile/message APIs. Components read the signals
 * and call the methods; mutations re-fetch to stay consistent with the backend.
 */
@Injectable({ providedIn: 'root' })
export class ProfileStore {
  private readonly profileApi = inject(ProfileApiService);
  private readonly messageApi = inject(MessageApiService);

  private readonly _profiles = signal<string[]>([]);
  private readonly _selectedProfile = signal<string | null>(null);
  private readonly _frames = signal<CanMessage[]>([]);
  private readonly _loading = signal(false);
  private readonly _error = signal<string | null>(null);

  readonly profiles = this._profiles.asReadonly();
  readonly selectedProfile = this._selectedProfile.asReadonly();
  readonly frames = this._frames.asReadonly();
  readonly loading = this._loading.asReadonly();
  readonly error = this._error.asReadonly();

  loadProfiles(): void {
    this._loading.set(true);
    this.profileApi.listProfiles().subscribe({
      next: (names) => {
        this._profiles.set(names);
        this._loading.set(false);
      },
      error: () => this.fail('Failed to load profiles'),
    });
  }

  selectProfile(name: string): void {
    this._selectedProfile.set(name);
    this.refreshFrames();
  }

  createProfile(name: string): Observable<TestProfile> {
    return this.profileApi.createProfile(name).pipe(
      tap((profile) => {
        this._profiles.update((list) => [...list, profile.name].sort());
        this.selectProfile(profile.name);
      }),
    );
  }

  addFrame(input: NewCanMessageInput): Observable<CanMessage> {
    const profile = this._selectedProfile();
    if (!profile) throw new Error('no profile selected');
    return this.messageApi.createMessage(profile, input).pipe(
      tap(() => this.refreshFrames()),
    );
  }

  deleteFrame(id: number): void {
    this.messageApi.deleteMessage(id).subscribe({
      next: () => this.refreshFrames(),
      error: () => this.fail('Failed to delete frame'),
    });
  }

  deleteProfile(name: string): void {
    this.profileApi.deleteProfile(name).subscribe({
      next: () => {
        this._profiles.update((list) => list.filter((n) => n !== name));
        if (this._selectedProfile() === name) {
          this._selectedProfile.set(null);
          this._frames.set([]);
        }
      },
      error: () => this.fail('Failed to delete profile'),
    });
  }

  clearError(): void {
    this._error.set(null);
  }

  private refreshFrames(): void {
    const profile = this._selectedProfile();
    if (!profile) {
      this._frames.set([]);
      return;
    }
    this._loading.set(true);
    this.messageApi.listMessages(profile).subscribe({
      next: (frames) => {
        this._frames.set(frames);
        this._loading.set(false);
      },
      error: () => this.fail('Failed to load frames'),
    });
  }

  private fail(message: string): void {
    this._error.set(message);
    this._loading.set(false);
  }
}
```

- [ ] **Step 2: Verify the build**

```bash
cd pythia-frontend && npm run build
```

Expected: build passes.

- [ ] **Step 3: Commit**

```bash
cd /home/caiod/NER/Pythia
git add pythia-frontend/src/app/state
git commit -m "feat(frontend): add signal-based ProfileStore facade"
```

---

## Task 4: Theme + Shell component

**Files:**
- Modify: `pythia-frontend/src/styles.scss` (beige/orange Material theme + base styles)
- Create: `pythia-frontend/src/app/components/shell/shell.ts`
- Create: `pythia-frontend/src/app/components/shell/shell.html`
- Create: `pythia-frontend/src/app/components/shell/shell.scss`
- Modify: `pythia-frontend/src/app/app.ts` + `app.html` to render `<app-shell>`

**Interfaces:**
- Consumes: nothing yet (child components are added empty here, filled in Tasks 5–7).
- Produces: `ShellComponent` (`app-shell`) — the branded layout with header, tab bar (CAN Frames active; Analog/GPIO disabled), and a content area with `<ng-content>` slots or direct child components. For this task the content area shows a placeholder; Tasks 5–7 replace it.

- [ ] **Step 1: Define the beige/orange Material theme**

Replace `pythia-frontend/src/styles.scss` with:

```scss
@use '@angular/material' as mat;

html {
  color-scheme: light;
  @include mat.theme((
    color: (
      theme-type: light,
      primary: mat.$orange-palette,
      tertiary: mat.$yellow-palette,
    ),
    typography: Roboto,
    density: 0,
  ));

  // Pythia beige surface palette.
  --pythia-bg: #f4ecd8;
  --pythia-surface: #fbf6e9;
  --pythia-border: #e3d7b8;
  --pythia-accent: #d97a2b;
  --pythia-text: #3a3220;
}

html, body { height: 100%; }
body {
  margin: 0;
  background: var(--pythia-bg);
  color: var(--pythia-text);
  font-family: Roboto, 'Helvetica Neue', sans-serif;
}
```

Note: if `mat.$orange-palette` is unavailable in the installed Material version, use `mat.$brown-palette` for primary and `mat.$orange-palette`/`mat.$amber-palette` for tertiary; the goal is a warm beige surface with an orange accent.

- [ ] **Step 2: Create the shell component class**

Create `pythia-frontend/src/app/components/shell/shell.ts`:

```typescript
import { Component } from '@angular/core';
import { MatTabsModule } from '@angular/material/tabs';
import { MatIconModule } from '@angular/material/icon';

/** Branded app frame: header, tab bar, and a content area for the CAN config. */
@Component({
  selector: 'app-shell',
  standalone: true,
  imports: [MatTabsModule, MatIconModule],
  templateUrl: './shell.html',
  styleUrl: './shell.scss',
})
export class ShellComponent {}
```

- [ ] **Step 3: Create the shell template**

Create `pythia-frontend/src/app/components/shell/shell.html`:

```html
<header class="pythia-header">
  <div class="brand">
    <mat-icon>hub</mat-icon>
    <div class="brand-text">
      <span class="brand-name">PYTHIA</span>
      <span class="brand-sub">TEST MODE</span>
    </div>
  </div>
  <div class="status"><span class="dot"></span> IDLE</div>
</header>

<nav class="tabs">
  <button class="tab active">CAN Frames</button>
  <button class="tab" disabled title="Coming soon">Analog Out</button>
  <button class="tab" disabled title="Coming soon">GPIO Out</button>
</nav>

<main class="content">
  <ng-content></ng-content>
</main>

<footer class="pythia-footer">
  <span>Pythia · HIL Test Mode</span>
  <span>SQLite profile store</span>
</footer>
```

- [ ] **Step 4: Create the shell styles**

Create `pythia-frontend/src/app/components/shell/shell.scss`:

```scss
:host { display: block; min-height: 100vh; }

.pythia-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 12px 24px; background: var(--pythia-surface);
  border-bottom: 1px solid var(--pythia-border);
}
.brand { display: flex; align-items: center; gap: 12px; }
.brand mat-icon { color: var(--pythia-accent); }
.brand-text { display: flex; flex-direction: column; line-height: 1.1; }
.brand-name { font-weight: 700; letter-spacing: 2px; color: var(--pythia-accent); }
.brand-sub { font-size: 11px; opacity: 0.7; letter-spacing: 1px; }
.status { display: flex; align-items: center; gap: 6px; font-size: 13px; opacity: 0.8; }
.status .dot { width: 8px; height: 8px; border-radius: 50%; background: #9aa06e; }

.tabs { display: flex; gap: 4px; padding: 0 24px; background: var(--pythia-surface);
  border-bottom: 1px solid var(--pythia-border); }
.tab { background: none; border: none; padding: 12px 16px; cursor: pointer;
  color: var(--pythia-text); opacity: 0.6; font-size: 14px; }
.tab.active { opacity: 1; border-bottom: 2px solid var(--pythia-accent); font-weight: 600; }
.tab:disabled { cursor: not-allowed; opacity: 0.35; }

.content { padding: 24px; max-width: 1200px; margin: 0 auto; }

.pythia-footer { display: flex; justify-content: space-between; padding: 12px 24px;
  font-size: 12px; opacity: 0.6; border-top: 1px solid var(--pythia-border); }
```

- [ ] **Step 5: Render the shell from the root component**

Set `pythia-frontend/src/app/app.ts` to import and use `ShellComponent`. Replace the class/imports so the component uses the shell:

```typescript
import { Component } from '@angular/core';
import { ShellComponent } from './components/shell/shell';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [ShellComponent],
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App {}
```

Replace `pythia-frontend/src/app/app.html` with a placeholder body inside the shell:

```html
<app-shell>
  <p>CAN frame configuration goes here.</p>
</app-shell>
```

(Note: the generated root component may be named `App` in `app.ts` and referenced in `main.ts`. Keep the existing class name; only adjust imports/template. If the generated files use `app.component.ts` naming, use those paths instead.)

- [ ] **Step 6: Verify the build and appearance**

```bash
cd pythia-frontend && npm run build
```

Expected: build passes. Then manual check:

```bash
cd pythia-frontend && npm start
```

Open `http://localhost:4200`. Expected: beige page with a "PYTHIA / TEST MODE" header, an orange accent, a tab bar with CAN Frames active and Analog/GPIO disabled, and the placeholder text. Stop the server (Ctrl-C).

- [ ] **Step 7: Commit**

```bash
cd /home/caiod/NER/Pythia
git add pythia-frontend/src/styles.scss pythia-frontend/src/app/components/shell pythia-frontend/src/app/app.ts pythia-frontend/src/app/app.html
git commit -m "feat(frontend): add Pythia-branded shell and beige/orange theme"
```

---

## Task 5: Profile selector + create-profile dialog

**Files:**
- Create: `pythia-frontend/src/app/components/new-profile-dialog/new-profile-dialog.ts`
- Create: `pythia-frontend/src/app/components/new-profile-dialog/new-profile-dialog.html`
- Create: `pythia-frontend/src/app/components/new-profile-dialog/new-profile-dialog.scss`
- Create: `pythia-frontend/src/app/components/profile-selector/profile-selector.ts`
- Create: `pythia-frontend/src/app/components/profile-selector/profile-selector.html`
- Create: `pythia-frontend/src/app/components/profile-selector/profile-selector.scss`

**Interfaces:**
- Consumes: `ProfileStore` (Task 3).
- Produces:
  - `NewProfileDialogComponent` — a `MatDialog` content component; on submit calls `store.createProfile(name)`, shows a `409` as an inline "name already exists" error, closes on success.
  - `ProfileSelectorComponent` (`app-profile-selector`) — a `mat-select` bound to `store.profiles()` and `store.selectedProfile()`, plus a "New profile…" button that opens the dialog and a delete button for the selected profile.

- [ ] **Step 1: Create the new-profile dialog class**

Create `pythia-frontend/src/app/components/new-profile-dialog/new-profile-dialog.ts`:

```typescript
import { Component, inject, signal } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { HttpErrorResponse } from '@angular/common/http';
import { MatButtonModule } from '@angular/material/button';
import { MatDialogModule, MatDialogRef } from '@angular/material/dialog';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { ProfileStore } from '../../state/profile.store';

/** Dialog for creating a new test profile. Resolves to the created name or null. */
@Component({
  selector: 'app-new-profile-dialog',
  standalone: true,
  imports: [FormsModule, MatDialogModule, MatFormFieldModule, MatInputModule, MatButtonModule],
  templateUrl: './new-profile-dialog.html',
  styleUrl: './new-profile-dialog.scss',
})
export class NewProfileDialogComponent {
  private readonly store = inject(ProfileStore);
  private readonly dialogRef = inject(MatDialogRef<NewProfileDialogComponent>);

  readonly name = signal('');
  readonly error = signal<string | null>(null);
  readonly saving = signal(false);

  submit(): void {
    const trimmed = this.name().trim();
    if (!trimmed) {
      this.error.set('Name is required');
      return;
    }
    this.saving.set(true);
    this.error.set(null);
    this.store.createProfile(trimmed).subscribe({
      next: () => this.dialogRef.close(trimmed),
      error: (err: HttpErrorResponse) => {
        this.saving.set(false);
        this.error.set(err.status === 409 ? 'A profile with that name already exists' : 'Failed to create profile');
      },
    });
  }

  cancel(): void {
    this.dialogRef.close(null);
  }
}
```

- [ ] **Step 2: Create the dialog template**

Create `pythia-frontend/src/app/components/new-profile-dialog/new-profile-dialog.html`:

```html
<h2 mat-dialog-title>New Test Profile</h2>
<mat-dialog-content>
  <mat-form-field appearance="outline" class="full-width">
    <mat-label>Profile name</mat-label>
    <input matInput [ngModel]="name()" (ngModelChange)="name.set($event)"
           (keyup.enter)="submit()" placeholder="PROFILE_001" />
  </mat-form-field>
  @if (error()) {
    <p class="error">{{ error() }}</p>
  }
</mat-dialog-content>
<mat-dialog-actions align="end">
  <button mat-button (click)="cancel()">Cancel</button>
  <button mat-flat-button color="primary" [disabled]="saving()" (click)="submit()">Create</button>
</mat-dialog-actions>
```

- [ ] **Step 3: Create the dialog styles**

Create `pythia-frontend/src/app/components/new-profile-dialog/new-profile-dialog.scss`:

```scss
.full-width { width: 100%; min-width: 320px; }
.error { color: #b3261e; font-size: 13px; margin: 4px 0 0; }
```

- [ ] **Step 4: Create the profile selector class**

Create `pythia-frontend/src/app/components/profile-selector/profile-selector.ts`:

```typescript
import { Component, inject } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatDialog } from '@angular/material/dialog';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatSelectModule } from '@angular/material/select';
import { NewProfileDialogComponent } from '../new-profile-dialog/new-profile-dialog';
import { ProfileStore } from '../../state/profile.store';

/** Profile dropdown plus create/delete actions. */
@Component({
  selector: 'app-profile-selector',
  standalone: true,
  imports: [MatFormFieldModule, MatSelectModule, MatButtonModule, MatIconModule],
  templateUrl: './profile-selector.html',
  styleUrl: './profile-selector.scss',
})
export class ProfileSelectorComponent {
  readonly store = inject(ProfileStore);
  private readonly dialog = inject(MatDialog);

  onSelect(name: string): void {
    this.store.selectProfile(name);
  }

  openCreate(): void {
    this.dialog.open(NewProfileDialogComponent);
  }

  deleteSelected(): void {
    const name = this.store.selectedProfile();
    if (name && confirm(`Delete profile "${name}" and all its frames?`)) {
      this.store.deleteProfile(name);
    }
  }
}
```

- [ ] **Step 5: Create the profile selector template**

Create `pythia-frontend/src/app/components/profile-selector/profile-selector.html`:

```html
<div class="selector-row">
  <mat-form-field appearance="outline">
    <mat-label>Profile</mat-label>
    <mat-select [value]="store.selectedProfile()" (selectionChange)="onSelect($event.value)">
      @for (name of store.profiles(); track name) {
        <mat-option [value]="name">{{ name }}</mat-option>
      }
    </mat-select>
  </mat-form-field>

  <button mat-flat-button color="primary" (click)="openCreate()">
    <mat-icon>add</mat-icon> New profile
  </button>

  @if (store.selectedProfile()) {
    <button mat-stroked-button (click)="deleteSelected()">
      <mat-icon>delete</mat-icon> Delete profile
    </button>
  }
</div>
```

- [ ] **Step 6: Create the profile selector styles**

Create `pythia-frontend/src/app/components/profile-selector/profile-selector.scss`:

```scss
.selector-row { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
mat-form-field { min-width: 240px; }
```

- [ ] **Step 7: Wire selector into the shell content and load profiles on start**

Update `pythia-frontend/src/app/app.ts` to load profiles on init and render the selector. Replace its contents:

```typescript
import { Component, OnInit, inject } from '@angular/core';
import { ShellComponent } from './components/shell/shell';
import { ProfileSelectorComponent } from './components/profile-selector/profile-selector';
import { ProfileStore } from './state/profile.store';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [ShellComponent, ProfileSelectorComponent],
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App implements OnInit {
  private readonly store = inject(ProfileStore);
  ngOnInit(): void {
    this.store.loadProfiles();
  }
}
```

Replace `pythia-frontend/src/app/app.html`:

```html
<app-shell>
  <app-profile-selector />
</app-shell>
```

- [ ] **Step 8: Verify the build**

```bash
cd pythia-frontend && npm run build
```

Expected: build passes.

- [ ] **Step 9: Commit**

```bash
cd /home/caiod/NER/Pythia
git add pythia-frontend/src/app/components/new-profile-dialog pythia-frontend/src/app/components/profile-selector pythia-frontend/src/app/app.ts pythia-frontend/src/app/app.html
git commit -m "feat(frontend): add profile selector and create-profile dialog"
```

---

## Task 6: CAN frame table

**Files:**
- Create: `pythia-frontend/src/app/components/can-frame-table/can-frame-table.ts`
- Create: `pythia-frontend/src/app/components/can-frame-table/can-frame-table.html`
- Create: `pythia-frontend/src/app/components/can-frame-table/can-frame-table.scss`
- Modify: `pythia-frontend/src/app/app.html` (render the table)
- Modify: `pythia-frontend/src/app/app.ts` (import the table)

**Interfaces:**
- Consumes: `ProfileStore` (Task 3), `formatCanId`/`bytesToNumber` (Task 1).
- Produces: `CanFrameTableComponent` (`app-can-frame-table`) — a `mat-table` over `store.frames()` with columns canId, format, content, mode, tStart, period, actions; delete button calls `store.deleteFrame(id)`.

- [ ] **Step 1: Create the table component class**

Create `pythia-frontend/src/app/components/can-frame-table/can-frame-table.ts`:

```typescript
import { Component, inject } from '@angular/core';
import { MatButtonModule } from '@angular/material/button';
import { MatChipsModule } from '@angular/material/chips';
import { MatIconModule } from '@angular/material/icon';
import { MatTableModule } from '@angular/material/table';
import { CanMessage } from '../../models/can.models';
import { bytesToNumber, formatCanId } from '../../utils/payload.util';
import { ProfileStore } from '../../state/profile.store';

/** Read-only table of the selected profile's CAN frames, with per-row delete. */
@Component({
  selector: 'app-can-frame-table',
  standalone: true,
  imports: [MatTableModule, MatChipsModule, MatIconModule, MatButtonModule],
  templateUrl: './can-frame-table.html',
  styleUrl: './can-frame-table.scss',
})
export class CanFrameTableComponent {
  readonly store = inject(ProfileStore);
  readonly columns = ['canId', 'format', 'content', 'mode', 'tStart', 'period', 'actions'];

  canId(frame: CanMessage): string {
    return formatCanId(frame.can_id);
  }

  content(frame: CanMessage): string {
    return bytesToNumber(frame.data).toString();
  }

  delete(frame: CanMessage): void {
    if (confirm(`Delete frame ${this.canId(frame)}?`)) {
      this.store.deleteFrame(frame.id);
    }
  }
}
```

- [ ] **Step 2: Create the table template**

Create `pythia-frontend/src/app/components/can-frame-table/can-frame-table.html`:

```html
@if (!store.selectedProfile()) {
  <p class="empty">Select or create a profile to configure CAN frames.</p>
} @else {
  <table mat-table [dataSource]="store.frames()" class="frame-table">
    <ng-container matColumnDef="canId">
      <th mat-header-cell *matHeaderCellDef>CAN ID</th>
      <td mat-cell *matCellDef="let f">{{ canId(f) }}</td>
    </ng-container>

    <ng-container matColumnDef="format">
      <th mat-header-cell *matHeaderCellDef>Format</th>
      <td mat-cell *matCellDef="let f">
        <span class="chip">{{ f.is_extended ? 'EXT' : 'STD' }}</span>
      </td>
    </ng-container>

    <ng-container matColumnDef="content">
      <th mat-header-cell *matHeaderCellDef>Content</th>
      <td mat-cell *matCellDef="let f">{{ content(f) }}</td>
    </ng-container>

    <ng-container matColumnDef="mode">
      <th mat-header-cell *matHeaderCellDef>Mode</th>
      <td mat-cell *matCellDef="let f">
        <span class="chip mode">{{ f.mode === 'broadcast' ? 'BCAST' : '1-SHOT' }}</span>
      </td>
    </ng-container>

    <ng-container matColumnDef="tStart">
      <th mat-header-cell *matHeaderCellDef>T-Start (ms)</th>
      <td mat-cell *matCellDef="let f">{{ f.offset_ms }}</td>
    </ng-container>

    <ng-container matColumnDef="period">
      <th mat-header-cell *matHeaderCellDef>Period (ms)</th>
      <td mat-cell *matCellDef="let f">{{ f.period_ms ?? '—' }}</td>
    </ng-container>

    <ng-container matColumnDef="actions">
      <th mat-header-cell *matHeaderCellDef></th>
      <td mat-cell *matCellDef="let f">
        <button mat-icon-button (click)="delete(f)" aria-label="Delete frame">
          <mat-icon>delete</mat-icon>
        </button>
      </td>
    </ng-container>

    <tr mat-header-row *matHeaderRowDef="columns"></tr>
    <tr mat-row *matRowDef="let row; columns: columns"></tr>
  </table>

  @if (store.frames().length === 0) {
    <p class="empty">No frames yet. Add one below.</p>
  }
}
```

- [ ] **Step 3: Create the table styles**

Create `pythia-frontend/src/app/components/can-frame-table/can-frame-table.scss`:

```scss
.frame-table { width: 100%; background: var(--pythia-surface); border: 1px solid var(--pythia-border); border-radius: 8px; }
.empty { opacity: 0.6; padding: 16px 0; }
.chip { display: inline-block; padding: 2px 8px; border-radius: 4px; font-size: 12px;
  background: var(--pythia-border); color: var(--pythia-text); }
.chip.mode { background: var(--pythia-accent); color: #fff; }
```

- [ ] **Step 4: Render the table in the shell content**

Update `pythia-frontend/src/app/app.ts` imports to include the table, and `app.html` to render it. In `app.ts`, add `CanFrameTableComponent` to the imports array and the import statement:

```typescript
import { CanFrameTableComponent } from './components/can-frame-table/can-frame-table';
// imports: [ShellComponent, ProfileSelectorComponent, CanFrameTableComponent]
```

Update `pythia-frontend/src/app/app.html`:

```html
<app-shell>
  <app-profile-selector />
  <app-can-frame-table />
</app-shell>
```

- [ ] **Step 5: Verify the build**

```bash
cd pythia-frontend && npm run build
```

Expected: build passes.

- [ ] **Step 6: Commit**

```bash
cd /home/caiod/NER/Pythia
git add pythia-frontend/src/app/components/can-frame-table pythia-frontend/src/app/app.ts pythia-frontend/src/app/app.html
git commit -m "feat(frontend): add CAN frame table with per-row delete"
```

---

## Task 7: Add-frame form

**Files:**
- Create: `pythia-frontend/src/app/components/add-frame-form/add-frame-form.ts`
- Create: `pythia-frontend/src/app/components/add-frame-form/add-frame-form.html`
- Create: `pythia-frontend/src/app/components/add-frame-form/add-frame-form.scss`
- Modify: `pythia-frontend/src/app/app.html` + `app.ts` (render the form)

**Interfaces:**
- Consumes: `ProfileStore` (Task 3), payload util (Task 1), models (Task 1).
- Produces: `AddFrameFormComponent` (`app-add-frame-form`) — a reactive form that builds a `NewCanMessageInput` and calls `store.addFrame(...)`. Period field visible+required only for broadcast.

- [ ] **Step 1: Create the form component class**

Create `pythia-frontend/src/app/components/add-frame-form/add-frame-form.ts`:

```typescript
import { Component, computed, inject, signal } from '@angular/core';
import { HttpErrorResponse } from '@angular/common/http';
import {
  FormControl, FormGroup, ReactiveFormsModule, Validators,
} from '@angular/forms';
import { MatButtonModule } from '@angular/material/button';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { CanFormat, CanMode, NewCanMessageInput } from '../../models/can.models';
import {
  MAX_EXT_ID, MAX_PAYLOAD, MAX_STD_ID, numberToBytes, parseCanId, parsePayload,
} from '../../utils/payload.util';
import { ProfileStore } from '../../state/profile.store';

/** Form to add a CAN frame to the selected profile. */
@Component({
  selector: 'app-add-frame-form',
  standalone: true,
  imports: [
    ReactiveFormsModule, MatFormFieldModule, MatInputModule,
    MatSelectModule, MatButtonModule, MatIconModule,
  ],
  templateUrl: './add-frame-form.html',
  styleUrl: './add-frame-form.scss',
})
export class AddFrameFormComponent {
  readonly store = inject(ProfileStore);
  readonly submitError = signal<string | null>(null);

  readonly form = new FormGroup({
    canId: new FormControl('', { nonNullable: true, validators: [Validators.required] }),
    format: new FormControl<CanFormat>('std', { nonNullable: true }),
    payload: new FormControl('0', { nonNullable: true, validators: [Validators.required] }),
    mode: new FormControl<CanMode>('oneshot', { nonNullable: true }),
    offsetMs: new FormControl(0, { nonNullable: true, validators: [Validators.required, Validators.min(0)] }),
    periodMs: new FormControl<number | null>(null),
  });

  readonly isBroadcast = signal(false);

  constructor() {
    this.form.controls.mode.valueChanges.subscribe((mode) => {
      const broadcast = mode === 'broadcast';
      this.isBroadcast.set(broadcast);
      const period = this.form.controls.periodMs;
      if (broadcast) {
        period.setValidators([Validators.required, Validators.min(1)]);
      } else {
        period.clearValidators();
        period.setValue(null);
      }
      period.updateValueAndValidity();
    });
  }

  submit(): void {
    this.submitError.set(null);
    if (!this.store.selectedProfile()) {
      this.submitError.set('Select a profile first');
      return;
    }
    if (this.form.invalid) {
      this.form.markAllAsTouched();
      return;
    }
    const v = this.form.getRawValue();

    let can_id: number;
    let data: number[];
    try {
      can_id = parseCanId(v.canId);
      const payload = parsePayload(v.payload);
      if (payload > MAX_PAYLOAD) throw new Error('Payload exceeds 8 bytes');
      data = numberToBytes(payload);
    } catch (e) {
      this.submitError.set((e as Error).message);
      return;
    }

    const is_extended = v.format === 'ext' ? 1 : 0;
    const maxId = is_extended ? MAX_EXT_ID : MAX_STD_ID;
    if (can_id < 0 || can_id > maxId) {
      this.submitError.set(`CAN id out of range for ${v.format.toUpperCase()} (max 0x${maxId.toString(16).toUpperCase()})`);
      return;
    }

    const input: NewCanMessageInput = {
      can_id,
      is_extended,
      data,
      mode: v.mode,
      offset_ms: v.offsetMs,
      period_ms: v.mode === 'broadcast' ? v.periodMs : null,
    };

    this.store.addFrame(input).subscribe({
      next: () => this.reset(),
      error: (err: HttpErrorResponse) => {
        this.submitError.set(
          err.status === 400 ? (err.error ?? 'Invalid frame')
          : err.status === 404 ? 'Profile not found'
          : 'Failed to add frame',
        );
      },
    });
  }

  private reset(): void {
    this.form.reset({ canId: '', format: 'std', payload: '0', mode: 'oneshot', offsetMs: 0, periodMs: null });
    this.isBroadcast.set(false);
  }
}
```

- [ ] **Step 2: Create the form template**

Create `pythia-frontend/src/app/components/add-frame-form/add-frame-form.html`:

```html
<section class="add-frame">
  <h3>Add Frame</h3>
  <form [formGroup]="form" (ngSubmit)="submit()" class="grid">
    <mat-form-field appearance="outline">
      <mat-label>CAN ID (hex)</mat-label>
      <input matInput formControlName="canId" placeholder="0x201" />
    </mat-form-field>

    <mat-form-field appearance="outline">
      <mat-label>Format</mat-label>
      <mat-select formControlName="format">
        <mat-option value="std">Standard</mat-option>
        <mat-option value="ext">Extended</mat-option>
      </mat-select>
    </mat-form-field>

    <mat-form-field appearance="outline">
      <mat-label>Payload (number)</mat-label>
      <input matInput formControlName="payload" inputmode="numeric" placeholder="0" />
    </mat-form-field>

    <mat-form-field appearance="outline">
      <mat-label>Mode</mat-label>
      <mat-select formControlName="mode">
        <mat-option value="oneshot">Single Shot</mat-option>
        <mat-option value="broadcast">Broadcast</mat-option>
      </mat-select>
    </mat-form-field>

    <mat-form-field appearance="outline">
      <mat-label>T-Start (ms)</mat-label>
      <input matInput type="number" formControlName="offsetMs" min="0" />
    </mat-form-field>

    @if (isBroadcast()) {
      <mat-form-field appearance="outline">
        <mat-label>Period (ms)</mat-label>
        <input matInput type="number" formControlName="periodMs" min="1" />
      </mat-form-field>
    }

    <div class="actions">
      <button mat-flat-button color="primary" type="submit" [disabled]="!store.selectedProfile()">
        <mat-icon>add</mat-icon> Add Frame
      </button>
    </div>
  </form>

  @if (submitError()) {
    <p class="error">{{ submitError() }}</p>
  }
</section>
```

- [ ] **Step 3: Create the form styles**

Create `pythia-frontend/src/app/components/add-frame-form/add-frame-form.scss`:

```scss
.add-frame { margin-top: 24px; padding: 16px; background: var(--pythia-surface);
  border: 1px solid var(--pythia-border); border-radius: 8px; }
.add-frame h3 { margin: 0 0 12px; letter-spacing: 1px; opacity: 0.8; }
.grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; align-items: start; }
.actions { display: flex; align-items: center; }
.error { color: #b3261e; font-size: 13px; margin: 8px 0 0; }
```

- [ ] **Step 4: Render the form in the shell content**

In `pythia-frontend/src/app/app.ts`, add `AddFrameFormComponent` to imports:

```typescript
import { AddFrameFormComponent } from './components/add-frame-form/add-frame-form';
// imports: [ShellComponent, ProfileSelectorComponent, CanFrameTableComponent, AddFrameFormComponent]
```

Update `pythia-frontend/src/app/app.html`:

```html
<app-shell>
  <app-profile-selector />
  <app-can-frame-table />
  <app-add-frame-form />
</app-shell>
```

- [ ] **Step 5: Verify the build**

```bash
cd pythia-frontend && npm run build
```

Expected: build passes.

- [ ] **Step 6: Commit**

```bash
cd /home/caiod/NER/Pythia
git add pythia-frontend/src/app/components/add-frame-form pythia-frontend/src/app/app.ts pythia-frontend/src/app/app.html
git commit -m "feat(frontend): add CAN frame creation form with conditional period"
```

---

## Task 8: Error snackbars + end-to-end verification

**Files:**
- Modify: `pythia-frontend/src/app/app.ts` (surface `store.error()` via `MatSnackBar`)

**Interfaces:**
- Consumes: `ProfileStore.error` signal (Task 3), `MatSnackBar`.
- Produces: user-visible snackbars for store-level errors; final verified app.

- [ ] **Step 1: Surface store errors as snackbars**

Update `pythia-frontend/src/app/app.ts` to react to the error signal. Replace its contents:

```typescript
import { Component, OnInit, effect, inject } from '@angular/core';
import { MatSnackBar } from '@angular/material/snack-bar';
import { ShellComponent } from './components/shell/shell';
import { ProfileSelectorComponent } from './components/profile-selector/profile-selector';
import { CanFrameTableComponent } from './components/can-frame-table/can-frame-table';
import { AddFrameFormComponent } from './components/add-frame-form/add-frame-form';
import { ProfileStore } from './state/profile.store';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [ShellComponent, ProfileSelectorComponent, CanFrameTableComponent, AddFrameFormComponent],
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App implements OnInit {
  private readonly store = inject(ProfileStore);
  private readonly snackBar = inject(MatSnackBar);

  constructor() {
    effect(() => {
      const err = this.store.error();
      if (err) {
        this.snackBar.open(err, 'Dismiss', { duration: 5000 });
        this.store.clearError();
      }
    });
  }

  ngOnInit(): void {
    this.store.loadProfiles();
  }
}
```

- [ ] **Step 2: Verify the build**

```bash
cd pythia-frontend && npm run build
```

Expected: build passes.

- [ ] **Step 3: End-to-end manual verification against the backend**

Start the backend (separate terminal), then the frontend:

```bash
# terminal 1
cd /home/caiod/NER/Pythia/pythia-backend && cargo run
# terminal 2
cd /home/caiod/NER/Pythia/pythia-frontend && npm start
```

Open `http://localhost:4200` and verify:
1. Existing profiles appear in the dropdown.
2. "New profile" creates a profile (and a duplicate name shows the inline 409 error).
3. Selecting a profile lists its frames (payload shown as a number).
4. "Add Frame" as Single Shot (no period field) inserts a row; switching Mode to Broadcast reveals a required Period field; adding a broadcast frame inserts a row with a period.
5. Deleting a frame removes its row; deleting a profile clears the selection.
6. Backend errors (e.g. invalid frame) show a snackbar.

Stop both servers.

- [ ] **Step 4: Commit**

```bash
cd /home/caiod/NER/Pythia
git add pythia-frontend/src/app/app.ts
git commit -m "feat(frontend): surface store errors via snackbar"
```

---

## Self-Review Notes

- **Spec coverage:** profile select/create/delete (Tasks 5), frame list (Task 6), frame add with conditional period + validation (Task 7), frame delete (Task 6), payload-as-number conversion (Tasks 1, 6, 7), beige/orange theme + Pythia branding + disabled Analog/GPIO tabs (Task 4), API contract incl. DELETEs (Task 2), signal facade with re-fetch (Task 3), error snackbars (Task 8). Tests intentionally omitted per spec.
- **Placeholders:** none — all steps contain concrete code/commands.
- **Type consistency:** method/signature names (`loadProfiles`, `selectProfile`, `createProfile`, `addFrame`, `deleteFrame`, `deleteProfile`, `clearError`; `numberToBytes`/`bytesToNumber`/`parsePayload`/`parseCanId`/`formatCanId`) are consistent across Tasks 1–8.
- **Naming caveat:** Angular 20 CLI may generate the root component as `App` in `app.ts`/`app.html` or as `AppComponent` in `app.component.ts`. Tasks assume `app.ts`/`App`; if the generated names differ, use the actual generated paths/names — the imports and templates are otherwise identical.
