# HealthTaki Backend

Rust API service for HealthTaki. It doesn't custody funds or touch the Stellar
network directly for payments — the [frontend](../healthTaki) already does
that via Freighter and Horizon. This service owns **provider identity**:
letting a medical provider prove they control a Stellar wallet and get a
session, then storing/serving their profile.

## Stack

- [Axum](https://github.com/tokio-rs/axum) — HTTP framework
- [SQLx](https://github.com/launchbadge/sqlx) — async Postgres access, compile-time-checked migrations at startup
- [jsonwebtoken](https://github.com/Keats/jsonwebtoken) — session tokens
- [ed25519-dalek](https://github.com/dalek-cryptography/curve25519-dalek) + [stellar-strkey](https://github.com/stellar/rs-stellar-strkey) — verifying wallet signatures

## How auth works: "Sign in with Stellar"

There is no password. A provider's identity *is* their Stellar wallet:

1. `POST /auth/challenge { stellar_public_key }` → server generates a
   one-time nonce and returns a human-readable `message` to sign.
2. The frontend has Freighter sign that exact string with
   [`signMessage`](https://github.com/stellar/freighter/blob/main/docs/api-reference.md)
   (this implements [SEP-53](https://github.com/stellar/stellar-protocol/blob/master/ecosystem/sep-0053.md):
   `signature = ed25519_sign(SHA256("Stellar Signed Message:\n" + message))`).
3. `POST /auth/verify { stellar_public_key, nonce, signature, display_name? }`
   → the server reconstructs the message from the nonce, verifies the
   signature against the claimed public key, and atomically consumes the
   challenge (so it can't be replayed). On first login this also creates the
   provider's profile row. Returns a JWT.
4. Send that JWT as `Authorization: Bearer <token>` to protected routes.

This means: no password database, no credential-stuffing surface, and the
same key the provider already uses to receive payments is what proves who
they are.

## Endpoints

| Method | Path              | Auth | Description                                   |
| ------ | ----------------- | ---- | ---------------------------------------------- |
| GET    | `/health`         | —    | Liveness check                                 |
| POST   | `/auth/challenge` | —    | Request a signable nonce for a wallet address  |
| POST   | `/auth/verify`    | —    | Verify a signed challenge, get a session JWT   |
| GET    | `/providers/me`   | JWT  | Fetch the authenticated provider's profile     |
| PATCH  | `/providers/me`   | JWT  | Update display name / email / specialty / etc. |

## Getting started

```bash
cp .env.example .env
docker compose up -d        # starts Postgres on localhost:5432
cargo run                   # runs migrations automatically, then listens on :8080
```

Generate a real `JWT_SECRET` for `.env` with `openssl rand -hex 32` — the
placeholder in `.env.example` is not safe to use as-is.

### Running tests

```bash
cargo test
```

`src/auth/stellar_sig.rs` includes a self-signed round-trip unit test for the
signature verification logic. The full challenge → sign → verify → protected
route flow was manually verified end-to-end against `@stellar/stellar-sdk`
(the library Freighter's `signMessage` is built on) during development.

## Project structure

```
src/
  main.rs               Router wiring, middleware, startup
  config.rs              Env-based configuration
  state.rs                Shared app state (DB pool, config)
  error.rs                 AppError -> HTTP response mapping
  auth/
    routes.rs              POST /auth/challenge, /auth/verify
    challenge.rs            Nonce generation + DB-backed challenge lifecycle
    stellar_sig.rs           SEP-53 signature verification
    jwt.rs                    Session token issuance/verification
    extractor.rs               Axum extractor requiring a valid Bearer token
  providers/
    routes.rs               GET/PATCH /providers/me
    repo.rs                   Postgres queries
    model.rs                   Provider row type
migrations/               SQL migrations (run automatically on startup)
```

## Configuration

See `.env.example` for all variables: `DATABASE_URL`, `JWT_SECRET`,
`JWT_EXPIRES_IN_SECONDS`, `AUTH_CHALLENGE_TTL_SECONDS`, `PORT`,
`CORS_ALLOWED_ORIGIN`, `STELLAR_NETWORK` (informational only).

## Next steps

This covers provider identity only. Natural extensions, in rough order of
what the frontend's "Next steps" section calls out:

- **Payment indexing/reconciliation** — a background worker polling Horizon
  (or a Soroban event stream) for payments to each provider's address,
  persisted so the API can serve reliable history/reporting instead of the
  frontend polling Horizon directly on every page load.
- **Appointments & invoicing** — generate payment requests tied to a
  provider + patient + amount, and mark them paid when a matching on-chain
  payment is indexed.
- **Patient records** — if this grows into handling PHI, treat that as a
  compliance-driven design decision (encryption at rest, access auditing),
  not a bolt-on.
