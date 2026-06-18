# spotprice-api — Mobile App Integration Guide

Instructions for the **iOS app repo** (separate repository). This document is
self-contained: hand it to the app's Claude Code / developer. The homelab side
(API, database, daily Nord Pool fetch, APNs sending) is already deployed.

The service exposes today's and tomorrow's **SE3** electricity spot prices and
sends an **APNs push** once tomorrow's prices are published (~13:00–14:00 CET,
fired at a randomized time in 13:30–14:00 local).

---

## 1. Base URL & auth

- **Base URL:** `https://homelab.k12n.com`
- **Auth:** Authelia OIDC. Every `/api/v1/spotprice/*` endpoint requires a
  **Bearer JWT access token** with the scope **`read:spotprice`**.
  - OIDC issuer: `https://auth.k12n.com`
  - The native-app client issues **RS256-signed JWT access tokens**; the API
    validates them against Authelia's JWKS (`https://auth.k12n.com/jwks.json`).
  - Send the token as `Authorization: Bearer <access_token>`.
  - The Traefik ingress only routes `/api/v1/spotprice/*` requests that carry an
    `Authorization: Bearer …` header. A request without it will **not** reach the
    service (you'll get a 404 from the ingress, not a 401) — always attach the token.

### OIDC client requirements

Your client must request the `read:spotprice` scope. The homelab user adds this
scope to your Authelia client (the native-app client, `client_id: homelab-macos`,
already supports the `homelab://oauth/callback` redirect scheme; confirm with the
homelab user whether you share that client or need a dedicated iOS one).

Use the standard **Authorization Code + PKCE** flow:
- `response_type=code`, `code_challenge_method=S256`
- scopes: `openid profile email offline_access read:spotprice`
- Discovery: `https://auth.k12n.com/.well-known/openid-configuration`

Store the access token and refresh it via `offline_access` (refresh token).

---

## 2. Endpoints

All return JSON. Times are **UTC** ISO-8601 (`TIMESTAMPTZ`); convert to local for
display. Prices come in two units per point: `price_per_kwh` (SEK/kWh) and
`price_ore_per_kwh` (öre/kWh).

### `GET /api/v1/spotprice/today`
Today's 24 hourly SE3 prices.

```json
{
  "delivery_area": "SE3",
  "currency": "SEK",
  "date": "2026-06-18",
  "source_updated_at": "2026-06-17T11:02:14Z",
  "min": { "time": "2026-06-18T01:00:00Z", "price_ore_per_kwh": 12.3 },
  "max": { "time": "2026-06-18T16:00:00Z", "price_ore_per_kwh": 187.0 },
  "prices": [
    { "time": "2026-06-18T00:00:00Z", "price_per_kwh": 0.231, "price_ore_per_kwh": 23.1 },
    { "time": "2026-06-18T01:00:00Z", "price_per_kwh": 0.123, "price_ore_per_kwh": 12.3 }
  ]
}
```

`min`/`max` and `source_updated_at` are `null` when no prices are stored yet.

### `GET /api/v1/spotprice/tomorrow`
Same shape as `/today`. Returns **`204 No Content`** until tomorrow's prices have
been fetched (typically available after ~14:00 local). Handle 204 as "not yet
published".

### `GET /api/v1/spotprice/latest`
Convenience combined view for the price screen:

```json
{
  "today": { /* PricesResponse, same shape as /today */ },
  "tomorrow": { /* PricesResponse, or null if not published yet */ }
}
```

---

## 3. Push notifications (APNs)

### Apple setup (already done on the homelab side)
- APNs **token-based** auth (`.p8` key). Key ID, Team ID and the key are already
  installed on the server.
- App **bundle id:** `com.homelab.app`
- The server is configured to send to both APNs **sandbox** and **production**;
  it picks the host based on the `environment` you register the token with.

### Register the device token
After obtaining notification permission and the APNs device token:

```
POST /api/v1/spotprice/devices
Authorization: Bearer <access_token>
Content-Type: application/json

{ "token": "<hex APNs device token>", "environment": "sandbox" }
```

- `environment`: **`"sandbox"`** for debug/Xcode builds, **`"production"`** for
  TestFlight/App Store builds. (Choosing the wrong one means Apple rejects the
  push with `BadDeviceToken`.)
- Returns **`201 Created`**. The call is idempotent on the token (re-registering
  refreshes it), so call it on every app start and whenever the token changes.
- `400 Bad Request` if the token is empty or `environment` is not one of
  `sandbox`/`production`.

### Unregister (optional, e.g. on logout)
```
DELETE /api/v1/spotprice/devices/{token}
Authorization: Bearer <access_token>
```
Returns `204 No Content`. You can only delete your own tokens.

### Push payload
The notification is pre-filled (Swedish). Title/body example:

- **Title:** `Elpriser imorgon (SE3)`
- **Body:** `Lägst 23 öre kl 03:00, högst 187 öre kl 18:00`

The APNs payload also carries a custom `spotprice` object for deep-linking to the
price screen:

```json
{
  "aps": {
    "alert": { "title": "Elpriser imorgon (SE3)", "body": "Lägst 23 öre kl 03:00, högst 187 öre kl 18:00" },
    "sound": "default"
  },
  "spotprice": {
    "date": "2026-06-19",
    "currency": "SEK",
    "min": { "ore_per_kwh": 23.0, "time": "2026-06-19T01:00:00+02:00" },
    "max": { "ore_per_kwh": 187.0, "time": "2026-06-19T16:00:00+02:00" }
  }
}
```

Read `userInfo["spotprice"]` in your notification handler to deep-link. The push
fires **once per publication** of tomorrow's prices (idempotent — no duplicates
on server retry/restart).

---

## 4. Implementation checklist

1. [ ] Add `read:spotprice` to the OIDC scopes your client requests (and confirm
       with the homelab user that the Authelia client grants it).
2. [ ] OIDC Authorization Code + PKCE login against `https://auth.k12n.com`;
       store + refresh the access token.
3. [ ] Request notification permission; register with APNs; obtain device token.
4. [ ] `POST /api/v1/spotprice/devices` with `{ token, environment }` on launch
       and on token change. Use `environment: "sandbox"` for debug builds.
5. [ ] Fetch `/today`, `/tomorrow` (handle 204), or `/latest`; render the price
       chart. Convert UTC times to local; display `price_ore_per_kwh` (öre/kWh).
6. [ ] Handle the push: title/body are pre-filled; use `userInfo["spotprice"]`
       to deep-link to the price screen.

---

## 5. Quick reference

| Item | Value |
|------|-------|
| Base URL | `https://homelab.k12n.com` |
| OIDC issuer | `https://auth.k12n.com` |
| Required scope | `read:spotprice` |
| Token type | RS256 JWT access token (Bearer) |
| Bundle id | `com.homelab.app` |
| APNs env (debug) | `sandbox` |
| Delivery area | `SE3` |
| Price units | `price_per_kwh` (SEK/kWh), `price_ore_per_kwh` (öre/kWh) |
| Timestamps | UTC ISO-8601 — convert to local |
