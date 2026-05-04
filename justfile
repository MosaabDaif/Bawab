# justfile — run `just <command>` inside the nix dev shell

# show available commands
default:
    @just --list

# run the Rust backend (auto-reloads on save)
backend:
    cd backend && watchexec -r -e rs -- cargo run

# run the SvelteKit frontend
frontend:
    cd frontend && npm run dev

# run both backend and frontend in parallel
dev:
    just backend & just frontend & wait

# run SQLite migrations
migrate:
    cd backend && sqlx migrate run

# prepare sqlx offline query cache (needed before cargo build without DB)
prepare:
    cd backend && cargo sqlx prepare

# format everything
fmt:
    cd backend && cargo fmt
    cd frontend && npx prettier --write src/

# lint
lint:
    cd backend && cargo clippy -- -D warnings
    cd frontend && npx eslint src/

# build for production
build:
    cd backend && cargo build --release
    cd frontend && npm run build

# build for production deployment (static frontend + release backend)
build-prod:
    cd frontend && PUBLIC_API_URL=/api npm run build
    cd backend && cargo build --release

# create a new DB from scratch and seed the admin user
db-reset:
    rm -f backend/library.db
    cd backend && sqlx migrate run
    @echo "DB reset. Admin: admin / library2024"
