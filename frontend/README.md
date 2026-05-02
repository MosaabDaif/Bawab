# Library System — Frontend

SvelteKit + TypeScript + Chart.js UI for the library attendance tracking system.

## Stack

| Package | Purpose |
|---|---|
| SvelteKit 2 | Full-stack Svelte framework |
| TypeScript | Type-safe code |
| Chart.js 4 | Analytics charts |
| Plain CSS | Mobile-first styling |

## Pages

| Route | Description |
|---|---|
| `/` | Student check-in / check-out (mobile-friendly) |
| `/admin` | Admin login form |
| `/dashboard` | Analytics, live occupancy, activity log |

## Setup

### 1. Enter the Nix dev shell

```bash
nix develop
```

### 2. Configure environment

```bash
cp .env.example .env
# Edit PUBLIC_API_URL if the backend runs on a different host/port
```

### 3. Install dependencies

```bash
npm install
```

### 4. Start the dev server

```bash
npm run dev
# — or from the project root —
just frontend
```

Frontend starts on **http://localhost:5173**.

## Building for production

```bash
npm run build          # outputs to build/
node build/index.js    # runs the Node.js server
```

## Environment variables

| Variable | Description |
|---|---|
| `PUBLIC_API_URL` | Base URL of the Rust backend (default: `http://localhost:3000`) |

## API client

All backend calls are in `src/lib/api.ts`.
Every function is typed and throws `Error('AUTH_REQUIRED')` on 401 responses so the dashboard can redirect to `/admin` automatically.
