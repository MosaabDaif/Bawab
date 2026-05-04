#!/usr/bin/env bash
set -euo pipefail

# Bawab deployment script — idempotent, run as root on Oracle Cloud VM
# Usage: sudo bash deploy/setup.sh

BAWAB_DIR="/opt/bawab"
BAWAB_USER="bawab"
WEB_ROOT="/var/www/bawab"

echo "=== Bawab Deployment ==="

# 1. Install system packages
echo "[1/7] Installing packages..."
if command -v dnf &>/dev/null; then
    dnf install -y nginx firewalld
elif command -v apt-get &>/dev/null; then
    apt-get update
    apt-get install -y nginx
fi

# 2. Create service user
echo "[2/7] Creating service user..."
if ! id "$BAWAB_USER" &>/dev/null; then
    useradd --system --shell /usr/sbin/nologin --home-dir "$BAWAB_DIR" "$BAWAB_USER"
fi

# 3. Create directories
echo "[3/7] Setting up directories..."
mkdir -p "$BAWAB_DIR" "$WEB_ROOT"

# 4. Copy backend binary
echo "[4/7] Installing backend..."
if [ -f "backend/target/release/backend" ]; then
    cp backend/target/release/backend "$BAWAB_DIR/backend"
    chmod 755 "$BAWAB_DIR/backend"
else
    echo "  WARNING: backend/target/release/backend not found. Build first with: just build-prod"
fi

# Copy migrations directory for sqlx
if [ -d "backend/migrations" ]; then
    cp -r backend/migrations "$BAWAB_DIR/"
fi

# 5. Copy frontend static files
echo "[5/7] Installing frontend..."
if [ -d "frontend/build" ]; then
    rm -rf "${WEB_ROOT:?}/"*
    cp -r frontend/build/* "$WEB_ROOT/"
else
    echo "  WARNING: frontend/build/ not found. Build first with: just build-prod"
fi

# 6. Set up env file if it doesn't exist
if [ ! -f "$BAWAB_DIR/.env" ]; then
    cp deploy/env.example "$BAWAB_DIR/.env"
    chmod 600 "$BAWAB_DIR/.env"
    echo "  IMPORTANT: Edit $BAWAB_DIR/.env with your secrets!"
fi

chown -R "$BAWAB_USER":"$BAWAB_USER" "$BAWAB_DIR"

# 7. Install and enable services
echo "[6/7] Configuring services..."
cp deploy/nginx/bawab.conf /etc/nginx/conf.d/bawab.conf

# Remove default nginx config if it exists
rm -f /etc/nginx/sites-enabled/default 2>/dev/null || true

cp deploy/systemd/bawab-backend.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable --now bawab-backend
systemctl enable --now nginx
systemctl reload nginx

# 7. Firewall (Oracle Linux / RHEL)
echo "[7/7] Configuring firewall..."
if command -v firewall-cmd &>/dev/null; then
    firewall-cmd --permanent --add-service=http 2>/dev/null || true
    firewall-cmd --permanent --add-service=https 2>/dev/null || true
    firewall-cmd --reload 2>/dev/null || true
fi

echo ""
echo "=== Deployment complete ==="
echo "  Backend:  systemctl status bawab-backend"
echo "  Frontend: $WEB_ROOT"
echo "  Env file: $BAWAB_DIR/.env"
echo ""
echo "Next steps:"
echo "  1. Edit $BAWAB_DIR/.env with real secrets"
echo "  2. sudo systemctl restart bawab-backend"
echo "  3. Set up TLS with certbot (see deploy/README.md)"
