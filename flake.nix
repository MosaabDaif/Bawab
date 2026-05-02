{
  description = "Library attendance system — Svelte + Rust dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rust-overlay.inputs.nixpkgs.follows = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Pin to stable Rust with the exact components we need
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"       # needed by rust-analyzer
            "rust-analyzer"  # LSP
            "clippy"         # linter
            "rustfmt"        # formatter
          ];
        };

      in {
        devShells.default = pkgs.mkShell {
          name = "library-system";

          buildInputs = [

            # ── Rust toolchain ───────────────────────────────────────────
            rustToolchain

            # sqlx-cli: run migrations and prepare offline query cache
            pkgs.sqlx-cli

            # ── Native libs required by Rust crates ──────────────────────
            pkgs.openssl        # jsonwebtoken, reqwest
            pkgs.pkg-config     # lets build.rs find openssl
            pkgs.sqlite         # sqlx sqlite backend

            # ── Node / Svelte toolchain ───────────────────────────────────
            pkgs.nodejs_20

            # ── Dev tools ────────────────────────────────────────────────
            pkgs.just           # task runner (like make, but nicer)
            pkgs.watchexec      # auto-restart backend on file changes
            pkgs.jq             # pretty-print API responses while testing
            pkgs.curl           # quick endpoint testing
            pkgs.git

          ];

          # ── Environment variables ─────────────────────────────────────
          shellHook = ''
            echo ""
            echo "  Library system dev shell"
            echo "  ─────────────────────────────────────────"
            echo "  Rust   : $(rustc --version)"
            echo "  Cargo  : $(cargo --version)"
            echo "  Node   : $(node --version)"
            echo "  npm    : $(npm --version)"
            echo "  sqlx   : $(sqlx --version)"
            echo ""
            echo "  Commands:"
            echo "    just backend   → run Rust API (port 3000)"
            echo "    just frontend  → run SvelteKit (port 5173)"
            echo "    just dev       → run both in parallel"
            echo "    just migrate   → run DB migrations"
            echo "    just fmt       → format Rust + JS"
            echo ""

            # Tell openssl-sys where to find OpenSSL
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig"
            export OPENSSL_DIR="${pkgs.openssl.dev}"
            export OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib"

            # SQLite path (used by sqlx)
            export SQLITE_DIR="${pkgs.sqlite}"

            # Keep cargo build artifacts outside the project if you want:
            # export CARGO_TARGET_DIR="$HOME/.cache/cargo-targets/library-system"
          '';
        };
      }
    );
}
