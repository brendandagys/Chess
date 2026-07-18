# AGENTS.md

## Project Overview

Full-stack serverless multiplayer chess application. React 19 + TypeScript frontend, Rust Lambda backend, WebSocket API, DynamoDB persistence. Deployed at `chess.brendandagys.com`.

## Repository Layout

- `frontend/` — React 19 SPA (Vite, TypeScript 5.7, plain CSS)
- `sam/rust_app/` — Rust Lambda functions (API Gateway WebSocket, DynamoDB)
- `sam/template.yaml` — AWS SAM infrastructure definition
- `extras/` — Design docs and reference material

## Build & Verify

### Frontend

```sh
cd frontend
npm install
npm run build    # tsc -b && vite build
npm run lint     # ESLint (strict + stylistic + React rules)
```

### Backend

```sh
cd sam/rust_app
cargo check      # Fast type/borrow checking
cargo clippy     # Lint
cd ..
sam build        # Full Lambda build (requires Docker + cargo-lambda)
```

Always run `cargo check` after Rust changes. Always run `npm run lint` and `npm run build` after frontend changes. There is no test suite currently.

## Architecture Rules

- **`helpers/`** contains pure logic only (board, engine, AI, opening book). No I/O, no AWS SDK calls.
- **`player_action_handlers/`** owns all I/O and side effects. Each file handles one player action.
- **`utils/`** wraps AWS SDK interactions (DynamoDB, API Gateway Management).
- **`types/`** contains Serde data structures only. No logic.
- Do not break this separation.

## Code Style

### TypeScript

- Use `@src/` path alias for all imports.
- Components use `React.FC<Props>` with destructured props.
- Plain CSS in `src/css/`, one file per component with matching name. No CSS-in-JS, no CSS modules.
- No Redux — state lives in hooks and context.

### Rust

- Async everywhere (Tokio runtime).
- Handlers return `Result<ApiGatewayProxyResponse, Error>`.
- All types use Serde with `snake_case` serialization.
- Board state is base64-encoded packed nibbles (4 bits/square). Do not change encoding without updating both frontend and backend.

## Key Conventions

- WebSocket messages: `{ route: "game", data: PlayerAction }`. Responses: `{ statusCode, connectionId, messages[], data }`.
- Frontend positions are 1-indexed `[rank, file]` tuples. UCI strings (e.g., `"a1a2"`) are converted at the boundary.
- API messages carry a type discriminator: `Error | Warning | Info | Success`.
- DynamoDB: GameTable keyed by `game_id`; UserTable keyed by `username` + `sk` with GSI on `connection_id`.
- Static assets (images, sounds) are re-exported through barrel `index.ts` files.

## What to Avoid

- Don't add state management libraries (Redux, Zustand, etc.).
- Don't introduce CSS-in-JS or CSS modules.
- Don't put I/O in `helpers/` or logic in `utils/`.
- Don't change the board encoding without a coordinated frontend + backend update.
- Don't add dependencies without justification.
