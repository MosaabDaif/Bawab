# Library System — Backend

Rust + Axum + SQLite API for the library attendance tracking system.

## Stack

| Crate | Purpose |
|---|---|
| `axum` 0.7 | Web framework |
| `sqlx` 0.7 | Async SQLite |
| `bcrypt` | Password hashing |
| `jsonwebtoken` | JWT admin auth |
| `tower-http` | CORS middleware |
| `chrono` | ISO-8601 timestamps |

## Setup

### 1. Enter the Nix dev shell

```bash
nix develop
```

### 2. Configure environment

```bash
cp .env.example .env
# Edit .env — change JWT_SECRET to a long random string
```

### 3. Run migrations & seed admin

```bash
just migrate
# Creates library.db and inserts admin user (admin / library2024)
```

### 4. Start the server

```bash
just backend          # auto-restarts on file changes (uses watchexec)
# — or —
cargo run             # single run
```

Server starts on **http://localhost:3000**.

## API Routes

| Method | Path | Auth | Description |
|---|---|---|---|
| `POST` | `/scan` | None | Student check-in / check-out |
| `POST` | `/auth/login` | None | Returns JWT |
| `GET` | `/inside` | JWT | Who is currently inside |
| `GET` | `/log` | JWT | Last 100 scans |
| `GET` | `/analytics/daily` | JWT | Unique visitors per day (last 7 days) |
| `GET` | `/analytics/peak` | JWT | Scan count by hour (today) |
| `GET` | `/analytics/weekday` | JWT | Avg visitors by day of week |
| `GET` | `/export/csv` | JWT | Download all scans as CSV |

### POST /scan

```json
// Request
{ "student_id": "12345" }

// Response
{ "action": "IN", "student_id": "12345" }
```

Rules:
- `student_id` must be 4–12 numeric digits
- If the last scan was `IN` → inserts `OUT`; otherwise → inserts `IN`
- Duplicate scans within 2 seconds are silently ignored

### POST /auth/login

```json
// Request
{ "username": "admin", "password": "library2024" }

// Response
{ "token": "<jwt>" }
```

### Protected routes

Pass the JWT in the `Authorization` header:

```
Authorization: Bearer <token>
```

## Environment Variables

| Variable | Description |
|---|---|
| `DATABASE_URL` | SQLite path, e.g. `sqlite://library.db` |
| `JWT_SECRET` | Secret key for signing JWTs (keep this safe!) |
| `ADMIN_PASSWORD` | Initial admin password (used on first run) |

## Useful commands

```bash
just migrate      # run pending migrations
just prepare      # generate sqlx offline query cache
just fmt          # cargo fmt
just lint         # cargo clippy
just build        # release build
just db-reset     # drop + recreate DB from scratch
```
