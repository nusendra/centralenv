# CentralEnv

Self-hosted, single-admin manager for `.env` variables. Run the server on a home box, expose it over Tailscale, manage projects and environments through a web UI, and pull values onto any device with a small Rust CLI.

```
                ┌──────────────────────────┐
   Browser  ──▶ │  SvelteKit web UI :5173  │
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

| Path        | What it is                                                                 |
| ----------- | -------------------------------------------------------------------------- |
| `server/`   | Rust + axum + sqlx + SQLite. AES-256-GCM at-rest encryption. Bearer auth.  |
| `web/`      | SvelteKit admin UI. Login, projects, environments, raw `.env` editor, API tokens. |
| `cli/`      | Rust CLI: `centralenv login`, `pull`, `run`.                               |
| `start.sh`  | One-shot dev bootstrap: generates `.env`, builds, runs both server + web.  |

---

## Security model

- **One admin user** (set at first run via `start.sh`), bcrypt-hashed.
- **Web sessions** — admin login returns a Bearer token; the SHA-256 hash is stored in `admin_sessions` with a 7-day expiry. Token lives in browser `localStorage`.
- **CLI tokens** — created from the **Tokens** page, scoped to specific projects, shown once on creation, bcrypt-hashed at rest.
- **Variable values** — encrypted with AES-256-GCM using `MASTER_KEY` (random 32 bytes, base64). The key never touches SQLite; it's loaded from `server/.env`.
- **Network** — there is no built-in TLS. Run the server only on a private network (Tailscale, VPN, or behind a reverse proxy). Anything reachable from the open internet should sit behind something that terminates TLS.

If `MASTER_KEY` is lost, all encrypted values are unrecoverable — back it up.

---

## Requirements

| Tool       | Min version | Notes                                  |
| ---------- | ----------- | -------------------------------------- |
| Rust       | 1.75+       | `rustup install stable`                |
| Bun        | 1.x         | for the web dev server (`bun install`) |
| SQLite CLI | any         | only needed for manual DB inspection   |
| OpenSSL    | any         | used by `start.sh` to generate the key |
| Tailscale  | optional    | recommended way to reach the server    |

---

## First-time setup (server host)

```sh
git clone <this-repo> centralenv
cd centralenv
./start.sh
```

`start.sh` will:

1. Generate `server/.env` with a random `MASTER_KEY` and prompt for an admin username + password.
2. Generate `web/.env` (`VITE_API_URL=http://localhost:3001`).
3. Build the server (`cargo build`).
4. Install web deps (`bun install`).
5. Run the API on `:3001` and the web UI on `:5173`.

Open `http://localhost:5173` and log in.

### Manual setup (if you don't want `start.sh`)

```sh
# server/.env
cat > server/.env <<EOF
DATABASE_URL=sqlite://centralenv.db
MASTER_KEY=$(openssl rand -base64 32)
ADMIN_USERNAME=admin
ADMIN_PASSWORD=changeme
BIND_ADDR=0.0.0.0:3001
RUST_LOG=centralenv_server=info,tower_http=info
EOF

# build & run server
(cd server && DATABASE_URL=sqlite:centralenv.db cargo run)

# in another terminal: web
(cd web && bun install && bun run dev --port 5173)
```

---

## Using the web UI

1. **Login** with the admin credentials.
2. **Projects** — create one per app, with a slug (`acme-api`).
3. Inside a project, create **environments** (`development`, `staging`, `production`).
4. Edit variables in **Table** mode (key/value rows) or **Raw** mode (paste a `.env` blob; it diffs and applies).
5. **Tokens** page — create an API token scoped to specific projects. The raw token is shown **once** — copy it.

---

## Installing the CLI on another device

The other device must be able to reach the server (e.g. on the same Tailscale tailnet).

### Option A — build from source

```sh
git clone <this-repo> centralenv
cd centralenv/cli
cargo build --release

# put it on PATH
sudo ln -sf "$PWD/target/release/centralenv" /usr/local/bin/centralenv
centralenv --help
```

### Option B — `cargo install` from a path

```sh
cargo install --path centralenv/cli
```

Either way, `centralenv` ends up on `$PATH`.

### Configure once

```sh
centralenv login \
  --url http://<tailscale-host-or-ip>:3001 \
  --token <token-copied-from-web-ui>
```

This writes a config to `$XDG_CONFIG_HOME/centralenv/config.toml` (macOS: `~/Library/Application Support/centralenv/config.toml`). Treat this file like a credential.

### Find the server's address

On the server host:

```sh
tailscale ip -4         # e.g. 100.101.102.103
tailscale status        # shows the MagicDNS hostname (e.g. "homeserver")
```

Either form works in `--url`:

```sh
centralenv login --url http://homeserver:3001 --token …
centralenv login --url http://100.101.102.103:3001 --token …
```

---

## Using the CLI

### `pull` — write a `.env` file

```sh
# Inside any project directory
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

## Operating the server long-term

### Run as a service (macOS launchd)

Create `~/Library/LaunchAgents/com.centralenv.server.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>            <string>com.centralenv.server</string>
  <key>WorkingDirectory</key> <string>/path/to/centralenv/server</string>
  <key>ProgramArguments</key>
  <array>
    <string>/path/to/centralenv/server/target/release/centralenv-server</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>DATABASE_URL</key>     <string>sqlite://centralenv.db</string>
    <key>MASTER_KEY</key>       <string>BASE64_KEY_HERE</string>
    <key>BIND_ADDR</key>        <string>0.0.0.0:3001</string>
  </dict>
  <key>RunAtLoad</key>         <true/>
  <key>KeepAlive</key>         <true/>
  <key>StandardOutPath</key>   <string>/tmp/centralenv-server.log</string>
  <key>StandardErrorPath</key> <string>/tmp/centralenv-server.err</string>
</dict>
</plist>
```

Then:

```sh
(cd server && cargo build --release)
launchctl load ~/Library/LaunchAgents/com.centralenv.server.plist
```

Note the admin user is seeded on first boot only; the launchd plist doesn't need `ADMIN_USERNAME` / `ADMIN_PASSWORD` after that.

### Backups

The only state worth backing up is:

```
server/centralenv.db        # the encrypted variable store
server/.env                 # contains MASTER_KEY — without this, the DB is useless
```

A nightly `cp` of both files to another disk / cloud bucket is enough.

### Production web build (optional)

The current setup runs the SvelteKit dev server. If you want a production build:

```sh
(cd web && bun install && bun run build)
# serve the contents of web/build/ behind any static server,
# or via Node:  node web/build
```

Make sure `VITE_API_URL` points at the public/Tailscale URL of the API at build time.

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
│   ├── migrations/          # sqlx migrations
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
└── start.sh
```

---

## API reference (short)

All endpoints are JSON.

### Auth

| Method | Path           | Auth   | Body                                     | Returns                          |
| ------ | -------------- | ------ | ---------------------------------------- | -------------------------------- |
| POST   | `/auth/login`  | none   | `{ "username", "password" }`             | `{ "username", "token" }`        |
| POST   | `/auth/logout` | admin  | —                                        | 204                              |
| GET    | `/auth/me`     | admin  | —                                        | 204                              |

### Projects / Environments / Variables (admin)

| Method | Path                                                         |
| ------ | ------------------------------------------------------------ |
| GET    | `/api/projects`                                              |
| POST   | `/api/projects`                                              |
| PUT    | `/api/projects/:id`                                          |
| DELETE | `/api/projects/:id`                                          |
| GET    | `/api/projects/:id/environments`                             |
| POST   | `/api/projects/:id/environments`                             |
| DELETE | `/api/projects/:id/environments/:env_id`                     |
| GET    | `/api/environments/:env_id/variables`                        |
| POST   | `/api/environments/:env_id/variables`                        |
| DELETE | `/api/environments/:env_id/variables/:key`                   |

### Tokens (admin)

| Method | Path                  |
| ------ | --------------------- |
| GET    | `/api/tokens`         |
| POST   | `/api/tokens`         |
| DELETE | `/api/tokens/:id`     |

### CLI fetch

| Method | Path                                            | Auth      |
| ------ | ----------------------------------------------- | --------- |
| GET    | `/api/projects/:slug/env?environment=<name>`    | CLI token |

Returns `{ KEY: "value", … }`.

---

## Troubleshooting

**Login works, but a refresh redirects me to `/login`.**
You're running an old server build. Restart it:
```sh
lsof -i :3001 -t | xargs kill
(cd server && cargo run)
```

**`/projects` page takes seconds to load.**
The server scans bcrypt hashes for every CLI token request. If you have many CLI tokens, this is slow. Web admin sessions use SHA-256 + indexed lookup and stay fast.

**`Unauthorized — check your token` from CLI.**
Either the token was deleted from the **Tokens** page, or the project slug isn't in that token's allowed-projects list. Recreate the token and re-run `centralenv login`.

**`MASTER_KEY must be exactly 32 bytes (base64-encoded)`.**
Regenerate: `openssl rand -base64 32` and put it in `server/.env`. **Replacing the key invalidates all stored variable values.**

**`failed to connect to server`** from a remote device.
Check Tailscale is up on both ends (`tailscale status`), the URL in `centralenv login` is reachable (`curl http://<host>:3001/`), and `BIND_ADDR=0.0.0.0:3001` (not `127.0.0.1`).

---

## License

Personal/internal use. No license file shipped — add your own before sharing publicly.
