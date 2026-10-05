# get_achievements

Suas conquistas da RetroAchievements (as do RetroArch), em Rust de ponta a ponta:

- **API** (raiz do projeto): axum, lê a RetroAchievements com a sua chave. Porta 3001.
- **Front** (`web/`): Leptos compilado pra WebAssembly, servido pelo Trunk. Porta 8080.

## Configuração (uma vez)

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
cp .env.example .env     # e preencha API_KEY e RA_USERNAME
```

## Rodar (dois terminais)

```bash
# 1) API, na raiz
cargo run

# 2) Front
cd web && trunk serve    # abre http://127.0.0.1:8080
```

O Trunk repassa `/api/*` para a API (veja `web/Trunk.toml`), então o front chama
só `/api/...`, sem CORS e sem expor a chave.

## Onde mexer

- `web/src/api.rs`: todas as requisições (`api::profile()`, `api::recent_games(Some(5))`, ...).
- `web/src/app.rs`: os componentes Leptos.
- `web/style.css`: o CSS.
