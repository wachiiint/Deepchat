# Deployment

Only the server is deployed. Clients are desktop apps that connect to it over raw TCP.

| | Cloud (main) | Local (backup) |
|---|---|---|
| Where | Railway, built from `Dockerfile` | A laptop on a phone hotspot |
| Client host : port | `nozomi.proxy.rlwy.net` : `35943` | server laptop IP : `7878` |
| Updates | Auto-redeploy on push to `feat/deploy` | `cargo run -p server` |

AI variables (`DEEPCHAT_AI_URL`, `DEEPCHAT_AI_MODEL`, `DEEPCHAT_AI_KEY`): see `.env.example`.

## Cloud: Railway setup (once)

1. railway.com → log in with GitHub.
2. Install the Railway GitHub App on the account that owns the repo: `https://github.com/apps/railway-app/installations/new` → select Deepchat. Without it, branches don't load.
3. New Project → Deploy from GitHub repo → Deepchat.
4. Settings → Source → branch `feat/deploy`.
5. Variables → add the 3 AI variables.
6. Settings → Networking → TCP Proxy → port `7878`. Railway shows the public `host:port`.
7. Deployments → logs show `listening on 0.0.0.0:7878` (first build ≈ 5 min).

## Local backup: phone hotspot

1. Server laptop + 2 client machines join the same hotspot.
2. Server laptop: `hostname -I` → its IP (e.g. `172.20.10.x`).
3. Fedora firewall: `sudo firewall-cmd --add-port=7878/tcp` (until reboot).
4. Repo root: `cargo run -p server`.
5. Each client: host = server IP, port = `7878`.

Client machines need the app: `pnpm tauri build` in `client/` (build Windows installers on Windows).
