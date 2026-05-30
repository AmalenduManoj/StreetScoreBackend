# Deploy Cricscore Backend with Docker

Multi-stage `Dockerfile` builds the Rust binary once in a builder image, then copies only the binary into a small Debian runtime image (~100MB).

## 1. Prerequisites

- Docker installed locally
- Supabase (or any Postgres) **connection string**
- Frontend URL for CORS (your Vite app after deploy)

Use Supabase **session pooler** (port **5432**) in production when possible:

```text
postgresql://postgres.[ref]:[password]@aws-0-[region].pooler.supabase.com:5432/postgres?sslmode=require
```

## 2. Test locally with Docker

From the `cricscorebackend` folder:

```bash
docker build -t cricscore-api .

docker run --rm -p 8080:8080 \
  -e DATABASE_URL="your-supabase-or-local-url" \
  -e ALLOWED_ORIGINS="http://localhost:5173,http://127.0.0.1:5173" \
  -e FRONTEND_RESET_PASSWORD_URL="http://localhost:5173/reset" \
  cricscore-api
```

Check:

```bash
curl http://127.0.0.1:8080/teams/get
```

Apply DB migrations manually in Supabase SQL editor (files in `migrations/`) before first use.

## 3. Environment variables

| Variable | Required | Example |
|----------|----------|---------|
| `DATABASE_URL` | Yes | Supabase Postgres URL |
| `PORT` | No (default `8080`) | Set by Railway/Render/Fly automatically |
| `HOST` | No (default `0.0.0.0`) | Leave default in containers |
| `ALLOWED_ORIGINS` | Yes in prod | `https://your-app.vercel.app,http://localhost:5173` |
| `FRONTEND_RESET_PASSWORD_URL` | For forgot password | `https://your-app.vercel.app/reset` |
| `SMTP_*` | For email reset | See README |

## 4. Deploy on Railway

1. Push this repo to GitHub.
2. [railway.app](https://railway.app) → **New Project** → **Deploy from GitHub** → select `cricscorebackend`.
3. Railway detects `Dockerfile` automatically.
4. **Variables** tab → add `DATABASE_URL`, `ALLOWED_ORIGINS`, etc.
5. **Settings** → generate a public domain (e.g. `https://cricscore-api-production.up.railway.app`).
6. In your frontend (Vercel/Netlify), set:

   ```env
   VITE_API_BASE_URL=https://your-railway-domain.up.railway.app
   ```

## 5. Deploy on Render (Docker Web Service)

Repo: [StreetScoreBackend](https://github.com/AmalenduManoj/StreetScoreBackend) (`cricscorebackend` locally). Root of the repo must contain `Dockerfile` at `./Dockerfile`.

### Option A — Blueprint (`render.yaml`)

1. Push `render.yaml` to GitHub.
2. [dashboard.render.com](https://dashboard.render.com) → **New** → **Blueprint**.
3. Connect the **StreetScoreBackend** repo → Render creates `cricscore-api`.
4. When prompted, set **secret** env vars: `DATABASE_URL`, `ALLOWED_ORIGINS`, `FRONTEND_RESET_PASSWORD_URL` (see `.env.example`).
5. Wait for the first Docker build (often **10–20 min** on free tier).
6. Open the service URL, e.g. `https://cricscore-api.onrender.com`.

### Option B — Manual Web Service (same result)

1. **New** → **Web Service** → connect GitHub repo **StreetScoreBackend**.
2. **Name**: `cricscore-api` (any name).
3. **Region**: pick closest to users / Supabase.
4. **Branch**: `main` (or your default).
5. **Root Directory**: leave blank (repo root = backend).
6. **Runtime**: **Docker**.
7. **Dockerfile Path**: `./Dockerfile`
8. **Docker Build Context Directory**: `.` (default)
9. **Instance type**: Free works for testing; builds are slow and the service sleeps after ~15 min idle.
10. **Environment** → add variables:

| Key | Value |
|-----|--------|
| `DATABASE_URL` | Supabase session pooler URL, port **5432**, `?sslmode=require` |
| `ALLOWED_ORIGINS` | Your frontend origin(s), comma-separated, no trailing slash |
| `FRONTEND_RESET_PASSWORD_URL` | `https://<frontend-host>/reset` |
| `RUST_LOG` | `info` (optional) |

Do **not** set `PORT` — Render injects it; the app reads `PORT` and binds `0.0.0.0`.

11. **Advanced** → **Health Check Path**: `/teams/get`
12. **Create Web Service** → wait for deploy green.
13. Smoke test:

```bash
curl https://cricscore-api.onrender.com/teams/get
```

### After the API is live

1. Run SQL in Supabase from `migrations/` if not already applied.
2. Deploy **Yourscore** (static site) on Render, Vercel, or Netlify.
3. Build frontend with the API URL:

```bash
cd Yourscore
VITE_API_BASE_URL=https://cricscore-api.onrender.com npm run build
```

4. Put the **exact** frontend URL in `ALLOWED_ORIGINS` on Render (e.g. `https://yourscore.onrender.com`) and **Redeploy** the API if you change CORS.
5. On the frontend host, set `VITE_API_BASE_URL` to the same Render API URL and rebuild.

### Render gotchas

- **Cold start**: free tier sleeps; first request after idle can take 30–60s.
- **Build timeout**: if Docker build fails on free tier, retry or use a paid instance for faster builds.
- **CORS errors**: origin must match exactly (`https` vs `http`, no trailing `/`).
- **DB errors**: use pooler port **5432**, not transaction pooler **6543**.

## 6. Deploy on Fly.io

```bash
cd cricscorebackend
fly launch --no-deploy
# Choose app name, region; use Dockerfile when asked

fly secrets set DATABASE_URL="..." ALLOWED_ORIGINS="https://your-frontend.app"
fly deploy
```

`fly.toml` will be created; ensure `internal_port` matches `8080`.

## 7. VPS (Docker only)

```bash
docker build -t cricscore-api .
docker run -d --name cricscore \
  --restart unless-stopped \
  -p 8080:8080 \
  --env-file /opt/cricscore/.env \
  cricscore-api
```

Put **Caddy** or **nginx** in front for HTTPS on `api.yourdomain.com`.

## 8. Frontend pairing

Yourscore (`Vite`) is deployed separately. Production build:

```bash
cd Yourscore
VITE_API_BASE_URL=https://your-api-host.example.com npm run build
```

Deploy `dist/` to Vercel, Netlify, or Cloudflare Pages.

## 9. Render: “Application exited early”

The container exits immediately if startup fails. Check **Logs** in the Render dashboard (not just the deploy summary).

| Log message | Fix |
|-------------|-----|
| `DATABASE_URL is not set` | Render → **Environment** → add `DATABASE_URL` (full Supabase URI) |
| `Failed to connect to the database` | Use **session pooler** port **5432**, append `?sslmode=require`; unpause Supabase; verify password |
| `Invalid DATABASE_URL` | No spaces in the URL; special characters in password must be URL-encoded |
| `Address already in use` / bind error | Remove any custom `PORT` you set — let Render inject `PORT` automatically |
| Build OK but instant exit, no log | Redeploy after pushing latest `Dockerfile`; set **Health Check Path** to `/health` |

**Required Render env vars**

```text
DATABASE_URL=postgresql://postgres.[ref]:[PASSWORD]@aws-0-[region].pooler.supabase.com:5432/postgres?sslmode=require
ALLOWED_ORIGINS=https://your-frontend.onrender.com
```

Do **not** set `PORT` manually on Render.

**Health check:** `/health` (no database). API routes like `/teams/get` still need a working DB.

## 10. Notes

- First Docker build can take **5–15 minutes** (Rust compile). Later builds are faster thanks to dependency caching.
- Do **not** commit `.env` with secrets; use the host’s secret manager.
- Change the hardcoded JWT secret in `src/auth/jwt.rs` before serious production use.
