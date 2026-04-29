#!/usr/bin/env bash
set -e

ROOT="$(cd "$(dirname "$0")" && pwd)"
SERVER_DIR="$ROOT/server"
WEB_DIR="$ROOT/web"
ENV_FILE="$SERVER_DIR/.env"

# ── 1. Bootstrap server .env ─────────────────────────────────────────────────
if [ ! -f "$ENV_FILE" ]; then
  echo "🔧 No server .env found — creating one..."

  MASTER_KEY=$(openssl rand -base64 32)

  read -p "   Admin username [admin]: " ADMIN_USERNAME
  ADMIN_USERNAME="${ADMIN_USERNAME:-admin}"

  while true; do
    read -s -p "   Admin password: " ADMIN_PASSWORD; echo
    read -s -p "   Confirm password: " CONFIRM; echo
    [ "$ADMIN_PASSWORD" = "$CONFIRM" ] && break
    echo "   ✗ Passwords don't match, try again."
  done

  cat > "$ENV_FILE" <<EOF
DATABASE_URL=sqlite://centralenv.db
MASTER_KEY=$MASTER_KEY
ADMIN_USERNAME=$ADMIN_USERNAME
ADMIN_PASSWORD=$ADMIN_PASSWORD
BIND_ADDR=0.0.0.0:3001
RUST_LOG=centralenv_server=info,tower_http=info
EOF

  echo "   ✓ .env created"
fi

# ── 2. Bootstrap web .env ────────────────────────────────────────────────────
WEB_ENV="$WEB_DIR/.env"
if [ ! -f "$WEB_ENV" ]; then
  echo "VITE_API_URL=http://localhost:3001" > "$WEB_ENV"
fi

# ── 3. Build server (if binary missing or source newer) ──────────────────────
BINARY="$SERVER_DIR/target/debug/centralenv-server"
if [ ! -f "$BINARY" ] || find "$SERVER_DIR/src" -newer "$BINARY" -name "*.rs" | grep -q .; then
  echo "🔨 Building server..."
  (cd "$SERVER_DIR" && DATABASE_URL=sqlite:centralenv.db cargo build 2>&1)
  echo "   ✓ Server built"
fi

# ── 4. Install web deps if needed ────────────────────────────────────────────
if [ ! -d "$WEB_DIR/node_modules" ]; then
  echo "📦 Installing web dependencies..."
  (cd "$WEB_DIR" && bun install --silent)
  echo "   ✓ Dependencies installed"
fi

# ── 5. Start both processes ───────────────────────────────────────────────────
echo ""
echo "🚀 Starting CentralEnv..."
echo "   Server → http://localhost:3001"
echo "   Web UI → http://localhost:5173"
echo ""
echo "   Press Ctrl+C to stop both."
echo ""

cleanup() {
  echo ""
  echo "Stopping..."
  kill "$SERVER_PID" "$WEB_PID" 2>/dev/null
  wait "$SERVER_PID" "$WEB_PID" 2>/dev/null
  exit 0
}
trap cleanup INT TERM

(cd "$SERVER_DIR" && ./target/debug/centralenv-server) &
SERVER_PID=$!

(cd "$WEB_DIR" && bun run dev --port 5173) &
WEB_PID=$!

wait "$SERVER_PID" "$WEB_PID"
