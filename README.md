# CentralEnv

Self-hosted, single-admin manager for `.env` variables. Run the server on a home box, expose it over a Cloudflare Tunnel or Tailscale, manage projects and environments through a web UI, and pull values onto any device with a small Rust CLI.

```
                ┌──────────────────────────┐
   Browser  ──▶ │  SvelteKit web UI :3002  │
                │  (admin only, login)     │
                └──────────┬───────────────┘
                           │ /auth /api  (Bearer)
                           ▼
                ┌──────────────────────────┐
                │  Rust + axum API :3001   │
                │  AES-GCM encryption      │
                │  SQLite at-rest          │
                └──────────┬───────────────┘
                           │ /api/projects/:slug/env (Bearer)
                           ▼
                ┌──────────────────────────┐
   Dev box  ──▶ │  centralenv CLI          │
                │  pull / run              │
                └──────────────────────────┘
```

---

## Components

| Path       | What it is                                                                        |
| ---------- | --------------------------------------------------------------------------------- |
| `server/`  | Rust + axum + sqlx + SQLite. AES-256-GCM at-rest encryption. Bearer auth.         |
| `web/`     | SvelteKit admin UI. Login, projects, environments, raw `.env` editor, API tokens. |
| `cli/`     | Rust CLI: `centralenv login`, `pull`, `run`.                                      |
| `start.sh` | One-shot dev bootstrap: generates `.env`, builds, runs both server + web.         |

---

## Security model

- **One admin user** (set via env vars on first boot), password is checked with bcrypt.
- **Web sessions** — admin login returns a Bearer token; the SHA-256 hash is stored in `admin_sessions` with a 7-day expiry. Token lives in browser `localStorage`.
- **CLI tokens** — created from the **Tokens** page, scoped to specific projects, shown once on creation, bcrypt-hashed at rest.
- **Variable values** — encrypted with AES-256-GCM using `MASTER_KEY` (random 32 bytes, base64). The key never touches SQLite; it's loaded from the environment.
- **Network** — there is no built-in TLS. Run the server only on a private network (Tailscale, VPN) or behind a reverse proxy / Cloudflare Tunnel that terminates TLS.

If `MASTER_KEY` is lost, all encrypted values are unrecoverable — back it up.

---

## Requirements

| Tool       | Min version | Notes                                  |
| ---------- | ----------- | -------------------------------------- |
| Rust       | 1.88+       | `rustup install stable`                |
| Bun        | 1.x         | for the web dev server (`bun install`) |
| SQLite CLI | any         | only needed for manual DB inspection   |
| OpenSSL    | any         | used by `start.sh` to generate the key |

---

## First-time setup (local dev)

```sh
git clone <this-repo> centralenv
cd centralenv
./start.sh
```

`start.sh` will:

1. Generate `server/.env` with a random `MASTER_KEY` and prompt for an admin username + password.
2. Generate `web/.env` (`PUBLIC_API_URL=http://localhost:3001`).
3. Build the server (`cargo build`).
4. Install web deps (`bun install`).
5. Run the API on `:3001` and the web UI on `:5173` (dev mode).

Open `http://localhost:5173` and log in.

### Manual setup

```sh
# server/.env
cat > server/.env <<EOF
DATABASE_URL=sqlite://centralenv.db
MASTER_KEY=$(openssl rand -base64 32 | tr -d '\n ')
ADMIN_USERNAME=admin
ADMIN_PASSWORD=changeme
BIND_ADDR=0.0.0.0:3001
RUST_LOG=centralenv_server=info,tower_http=info
EOF

# build & run server
(cd server && DATABASE_URL=sqlite:centralenv.db cargo run)

# in another terminal: web
(cd web && bun install && bun run dev)
```

---

## Using the web UI

1. **Login** with the admin credentials.
2. **Projects** — create one per app, with a slug (`acme-api`).
3. Inside a project, create **environments** (`development`, `staging`, `production`).
4. Edit variables in **Table** mode (key/value rows) or **Raw** mode (paste a `.env` blob; it diffs and applies).
5. **Tokens** page — create an API token scoped to specific projects. The raw token is shown **once** — copy it.

---

## Installing the CLI

### Option A — download a prebuilt binary (fastest)

Pick the right archive from the latest [GitHub Release](https://github.com/nusendra/centralenv/releases/latest):

```sh
# macOS — Apple Silicon (M1/M2/M3/M4)
curl -L https://github.com/nusendra/centralenv/releases/latest/download/centralenv-macos-arm64.tar.gz \
  | tar xz
sudo mv centralenv-macos-arm64 /usr/local/bin/centralenv

# macOS — Intel
curl -L https://github.com/nusendra/centralenv/releases/latest/download/centralenv-macos-amd64.tar.gz \
  | tar xz
sudo mv centralenv-macos-amd64 /usr/local/bin/centralenv

# Linux x86_64
curl -L https://github.com/nusendra/centralenv/releases/latest/download/centralenv-linux-amd64.tar.gz \
  | tar xz
sudo mv centralenv-linux-amd64 /usr/local/bin/centralenv

# Linux arm64 (e.g. Raspberry Pi)
curl -L https://github.com/nusendra/centralenv/releases/latest/download/centralenv-linux-arm64.tar.gz \
  | tar xz
sudo mv centralenv-linux-arm64 /usr/local/bin/centralenv
```

Each release also ships a `.sha256` file for verification:

```sh
shasum -a 256 -c centralenv-macos-arm64.tar.gz.sha256
```

### Option B — build from source

```sh
git clone <this-repo> centralenv
cd centralenv/cli
cargo build --release
sudo ln -sf "$PWD/target/release/centralenv" /usr/local/bin/centralenv
```

### Configure once

```sh
centralenv login \
  --url https://<your-api-domain> \
  --token <token-copied-from-web-ui>
```

Config is written to `$XDG_CONFIG_HOME/centralenv/config.toml` (macOS: `~/Library/Application Support/centralenv/config.toml`). Treat this file like a credential.

---

## Using the CLI

### `pull` — write a `.env` file

```sh
centralenv pull acme-api -e production           # writes ./.env
centralenv pull acme-api -e development -o .env.dev
```

### `run` — inject without touching disk

```sh
centralenv run acme-api -e development -- npm run dev
centralenv run acme-api -e production  -- ./bin/server
centralenv run acme-api -- bash         # spawn a shell with the vars set
```

Anything after `--` is the command and its arguments. The vars exist only in that subprocess's environment.

---

## Deploying with Docker / Coolify

GitHub Actions builds and pushes prebuilt images to **GHCR** on every push to `master`. Coolify pulls these — no compilation on your home server.

| Image                                       | Platforms                    |
| ------------------------------------------- | ---------------------------- |
| `ghcr.io/nusendra/centralenv-server:latest` | linux/amd64, linux/arm64     |
| `ghcr.io/nusendra/centralenv-web:latest`    | linux/amd64, linux/arm64     |

`PUBLIC_API_URL` is read at **runtime** by the web container — changing the API URL never requires a rebuild.

### Coolify (recommended)

**1. Make GHCR packages accessible**

After the first CI run, go to GitHub → your profile → **Packages** → open each package (`centralenv-server`, `centralenv-web`) → Settings → set **Visibility: Public**. Or add a GitHub PAT as a registry credential in Coolify.

**2. Create the project**

- New Resource → **Docker Compose** → point at this Git repo (`master`).
- Coolify detects `docker-compose.yml` (uses `image:`, no build step on the host).

**3. Assign domains**

- `server` → e.g. `https://centralenv-api.example.com`
- `web` → e.g. `https://centralenv.example.com`

**4. Set environment variables**

| Key              | Value                                     | Notes                                               |
| ---------------- | ----------------------------------------- | --------------------------------------------------- |
| `MASTER_KEY`     | see generation command below              | Mark as Secret. Back it up — losing it bricks the DB. |
| `ADMIN_USERNAME` | `admin`                                   | First-boot seed only.                               |
| `ADMIN_PASSWORD` | `<your password>`                         | Mark as Secret. First-boot seed only.               |
| `PUBLIC_API_URL` | `https://centralenv-api.example.com`      | Runtime — no rebuild needed on change.              |
| `SERVER_PORT`    | `3001`                                    | Optional host port override.                        |
| `WEB_PORT`       | `3002`                                    | Optional host port override.                        |
| `RUST_LOG`       | `centralenv_server=info,tower_http=info`  | Optional.                                           |

Generate `MASTER_KEY` (must have no spaces or newlines):

```sh
openssl rand -base64 32 | tr -d '\n '
```

Optional: pin to a specific image tag instead of `:latest`:

```
SERVER_IMAGE=ghcr.io/nusendra/centralenv-server:v0.1.0
WEB_IMAGE=ghcr.io/nusendra/centralenv-web:v0.1.0
```

**5. Deploy**

Click Deploy. Coolify pulls both images and starts the services. Volume `centralenv-data` persists the SQLite DB across redeploys.

**6. Point the CLI at it**

```sh
centralenv login \
  --url https://centralenv-api.example.com \
  --token <token-from-Tokens-page>
```

### Cloudflare Tunnel (bypassing Traefik)

If you run a Cloudflare Tunnel in front of Coolify, **point the tunnel directly at the container host ports** rather than through Traefik on port 80. Traefik's HTTP entrypoint redirects to HTTPS, which creates a redirect loop when combined with Cloudflare's TLS termination.

In `/etc/cloudflared/config.yml` on your home server:

```yaml
ingress:
  - hostname: centralenv.example.com
    service: http://127.0.0.1:3002        # web container host port
    originRequest:
      hostHeader: centralenv.example.com
  - hostname: centralenv-api.example.com
    service: http://127.0.0.1:3001        # server container host port
    originRequest:
      hostHeader: centralenv-api.example.com
  - service: http_status:404
```

Then restart: `sudo systemctl restart cloudflared`

### Local Docker

```sh
cp .env.example .env
# edit: set MASTER_KEY, ADMIN_PASSWORD
# set PUBLIC_API_URL=http://localhost:3001 if using browser on same machine

docker compose pull
docker compose up -d
# server → http://localhost:3001
# web    → http://localhost:3002
```

### Backups

```sh
# Docker volume backup
docker run --rm -v centralenv_centralenv-data:/data alpine tar czf - /data > backup.tgz
```

Back up `MASTER_KEY` separately. Without it, the SQLite contents are unrecoverable.

### Tagged releases

Push a tag to produce versioned Docker images and CLI binaries:

```sh
git tag v0.1.0 && git push origin v0.1.0
```

This produces:

- `ghcr.io/nusendra/centralenv-server:v0.1.0` and `:latest`
- `ghcr.io/nusendra/centralenv-web:v0.1.0` and `:latest`
- A GitHub Release with CLI binaries: `macos-amd64`, `macos-arm64`, `linux-amd64`, `linux-arm64` (each as `*.tar.gz` + `.sha256`)

---

## Operating the server long-term

### Backups (non-Docker)

The only state worth backing up:

```
server/centralenv.db   # the encrypted variable store
server/.env            # contains MASTER_KEY — without this, the DB is useless
```

A nightly `cp` of both files to another disk / cloud bucket is enough.

---

## Project structure

```
centralenv/
├── cli/
│   ├── src/
│   │   ├── main.rs          # clap CLI: login / pull / run
│   │   ├── client.rs        # HTTP client, reqwest + bearer auth
│   │   └── config.rs        # config.toml load/save
│   └── Cargo.toml
├── server/
│   ├── migrations/
│   │   ├── 0001_initial.sql
│   │   └── 0002_admin_sessions.sql
│   ├── src/
│   │   ├── main.rs          # axum app, CORS, migrations, seed
│   │   ├── auth.rs          # TokenAuth + AdminSession extractors
│   │   ├── crypto.rs        # AES-256-GCM encrypt/decrypt
│   │   ├── db.rs            # admin user seeding
│   │   ├── error.rs         # AppError
│   │   ├── models.rs
│   │   └── routes/
│   │       ├── auth.rs      # /auth/login, /auth/logout, /auth/me
│   │       ├── projects.rs
│   │       ├── environments.rs
│   │       ├── variables.rs
│   │       └── tokens.rs
│   └── Cargo.toml
├── web/
│   ├── src/
│   │   ├── lib/
│   │   │   ├── api.ts       # fetch wrapper, attaches Bearer
│   │   │   └── auth.ts      # localStorage-backed authed store
│   │   └── routes/
│   │       ├── +layout.svelte
│   │       ├── +page.svelte
│   │       ├── login/
│   │       ├── projects/
│   │       └── tokens/
│   └── package.json
├── docker-compose.yml
├── .env.example
└── start.sh
```

---

## API reference

All endpoints return JSON.

### Auth

| Method | Path           | Auth  | Body                            | Returns                   |
| ------ | -------------- | ----- | ------------------------------- | ------------------------- |
| POST   | `/auth/login`  | none  | `{ "username", "password" }`   | `{ "username", "token" }` |
| POST   | `/auth/logout` | admin | —                               | 204                       |
| GET    | `/auth/me`     | admin | —                               | 204                       |

### Projects / Environments / Variables (admin Bearer)

| Method | Path                                          |
| ------ | --------------------------------------------- |
| GET    | `/api/projects`                               |
| POST   | `/api/projects`                               |
| PUT    | `/api/projects/:id`                           |
| DELETE | `/api/projects/:id`                           |
| GET    | `/api/projects/:id/environments`              |
| POST   | `/api/projects/:id/environments`              |
| DELETE | `/api/projects/:id/environments/:env_id`      |
| GET    | `/api/environments/:env_id/variables`         |
| POST   | `/api/environments/:env_id/variables`         |
| DELETE | `/api/environments/:env_id/variables/:key`    |

### Tokens (admin Bearer)

| Method | Path              |
| ------ | ----------------- |
| GET    | `/api/tokens`     |
| POST   | `/api/tokens`     |
| DELETE | `/api/tokens/:id` |

### CLI fetch (CLI token)

| Method | Path                                         |
| ------ | -------------------------------------------- |
| GET    | `/api/projects/:slug/env?environment=<name>` |

Returns `{ KEY: "value", … }`.

---

## Troubleshooting

**`MASTER_KEY` error on Coolify / "Invalid symbol" or "must be 32 bytes".**
The key must have no spaces or newlines. Always generate it with:
```sh
openssl rand -base64 32 | tr -d '\n '
```
Pasting from a terminal that wraps the output adds a newline. If you change the key, all stored variable values become unrecoverable.

**SQLite "unable to open database file" in Docker.**
The `centralenv-data` volume must be writable by the container. The server container runs as root — if you manually created the volume directory and it's owned by another user, fix it:
```sh
# on the Docker host
sudo chown -R root:root /var/lib/docker/volumes/centralenv_centralenv-data
```

**Server container shows "unhealthy" in `docker ps`.**
The healthcheck requires `curl`. Make sure you're running the latest image (built after the `curl` fix was merged). Pull and redeploy:
```sh
docker compose pull && docker compose up -d
```

**`centralenv.example.com` shows a redirect loop.**
If using a Cloudflare Tunnel pointed at Traefik port 80, Traefik redirects HTTP→HTTPS, which loops. Fix: point cloudflared directly at the container's host port (`3002` for web, `3001` for API) — see the Cloudflare Tunnel section above.

**`Unauthorized — check your token` from CLI.**
Either the token was deleted from the **Tokens** page, or the project slug isn't in that token's allowed-projects list. Recreate the token and re-run `centralenv login`.

**`failed to connect to server` from a remote device.**
Check that the URL in `centralenv login` is reachable (`curl https://<host>/auth/me` should return 401), and that `BIND_ADDR=0.0.0.0:3001` (not `127.0.0.1`).

---

## License

Personal/internal use. No license file shipped — add your own before sharing publicly.
