# Deploying Bawab to Oracle Cloud Free Tier

## Architecture

```
Internet --> nginx (:80/:443)
               ├── /api/*   --> Rust backend (127.0.0.1:3000)
               └── /*       --> static files (SvelteKit build)
```

No Docker. Bare-metal with systemd + nginx. No Node.js in production.

## 1. Provision Oracle Cloud VM

1. Create an **Always Free** ARM instance (Ampere A1, up to 4 OCPU / 24 GB RAM)
2. Choose **Oracle Linux 9** or **Ubuntu 22.04**
3. Download your SSH key during creation

### Security Lists (Oracle Cloud Console)

Add ingress rules to the VCN's default security list:

| Port | Protocol | Source     | Purpose |
|------|----------|-----------|---------|
| 22   | TCP      | Your IP   | SSH     |
| 80   | TCP      | 0.0.0.0/0 | HTTP    |
| 443  | TCP      | 0.0.0.0/0 | HTTPS   |

## 2. Server Setup

SSH into the VM:

```bash
ssh opc@<public-ip>
```

### Install Rust (for building on the server)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Install Node.js (for building frontend only)

```bash
curl -fsSL https://rpm.nodesource.com/setup_20.x | sudo bash -
sudo dnf install -y nodejs
```

## 3. Build

Clone the repo and build:

```bash
git clone <your-repo-url> bawab
cd bawab
just build-prod
```

Or build manually:

```bash
cd frontend && npm ci && npm run build && cd ..
cd backend && cargo build --release && cd ..
```

## 4. Deploy

```bash
sudo bash deploy/setup.sh
```

Then edit the production environment file:

```bash
sudo nano /opt/bawab/.env
```

Set real values for `JWT_SECRET` and `ADMIN_PASSWORD`, then restart:

```bash
sudo systemctl restart bawab-backend
```

## 5. Verify

```bash
# Backend responds
curl http://localhost/api/scan

# Frontend loads
curl -s http://localhost/ | head -5
```

## 6. TLS with Certbot

Point your domain's DNS A record to the VM's public IP, then:

```bash
# Oracle Linux / RHEL
sudo dnf install -y certbot python3-certbot-nginx

# Ubuntu
sudo apt install -y certbot python3-certbot-nginx

# Get certificate
sudo certbot --nginx -d yourdomain.com

# Auto-renew is set up automatically; verify with:
sudo certbot renew --dry-run
```

## Updating

After pulling new code:

```bash
just build-prod
sudo bash deploy/setup.sh
sudo systemctl restart bawab-backend
```
