# Phone companion API (v1)

The desktop app serves a mobile web page to phones, which have no app and no client certificate. A phone gets in only through a one-time QR token shown by the desktop. Without a valid session, any request is treated like any other browser request: `/api/m/*` returns `401`, and pages redirect to `/download`.

## Entry
- The desktop shows a QR code for `https://<lan-ip>:<port>/m/#t=<token>`.
  - The token is single-use, random, 32+ bytes, base64url-encoded.
  - It expires after 5 minutes.
- `GET /m/` and `GET /m/<asset>` serve the static mobile app (SPA), embedded in the binary. These paths are public; the token stays in the URL fragment, so it never reaches server logs.
- The page reads the token from `location.hash`, removes it from the URL (`history.replaceState`), and calls `POST /api/m/session`.

## Session
- `POST /api/m/session`, body `{ "token": "..." }`.
  - Response `200 { "deviceName": "<desktop name>", "sessionId": "..." }`. It also sets the cookie `muniko_session=<id>; HttpOnly; Secure; SameSite=Strict; Path=/`.
  - A token that is unknown, used or expired gets `401`.
- Sessions last 12 h (sliding). The desktop can revoke a session at any time, after which every call returns `401`.
- `GET /api/m/session`: `200 { "deviceName", "sessionId" }` if the session is valid, otherwise `401`. The page uses it on reload to reuse an existing session.
- `DELETE /api/m/session` disconnects the phone (logout).

All endpoints below need a valid session cookie (otherwise `401`).

## Files phone → desktop
- `POST /api/m/upload`
  - Body: the raw file bytes, streamed.
  - Headers: `x-muniko-file-name` (URI-encoded UTF-8) and `content-length`.
  - The file is saved to the desktop download folder, sanitized and given a unique name. The desktop publishes `FileReceived { from_device_id: "phone:<sessionId>", path }`.
  - Responses: `201 { "name": "<saved name>" }`, `400` (bad name or empty), `413` (over the limit, default 4 GiB).

## Files desktop → phone (inbox)
- `GET /api/m/inbox`: `200 [{ "id", "name", "size", "addedAt" }]`, newest first.
- `GET /api/m/inbox/{id}`: streams the file with `Content-Disposition: attachment` and supports `Range`.
- Items expire after 24 h or when the session ends.

## Text
- `POST /api/m/text`, body `{ "text": "..." }` (max 1 MiB).
  - The text is placed on the desktop clipboard, and the desktop publishes `ClipboardReceived { from_device_id: "phone:<sessionId>", text }`.
  - Responses: `204`, `400` (empty), `413` (too large).
- `GET /api/m/text`: `200 { "text": "...", "sentAt": <unix> }` with the latest text the desktop shared to this phone, or `204` if there is none.

## Live updates
- `GET /api/m/events`: a Server-Sent Events stream.
  - Events: `inbox` (the inbox changed), `text` (new text shared), `revoked` (the session ended).
  - The page re-fetches on each event. If SSE is unavailable, it polls every 5 s.

## Board
The phone sees the desktop's Network Board. Anything the phone posts is published by the desktop under the desktop's own device identity.
- `GET /api/m/board` returns `200 { "board": { "id", "name", "mode" } | null, "items": [BoardItemView] }`, newest first.
  - `BoardItemView` (camelCase) is `{ id, authorId, authorName, createdAt, mine, heldLocally, available, kind: { type: "text", text } | { type: "file", name, size, sha256 } }`.
- `POST /api/m/board/text`, body `{ "text": "..." }`. Responses: `201 BoardItemView`, `400` (empty), `413` (too large), `409` (no board yet).
- `POST /api/m/board/files`: the raw body plus an `x-muniko-file-name` header, the same as `/api/m/upload`. The desktop stores the file and posts it to the board. Responses: `201 BoardItemView`, `400`, `409`, `413`.
- `GET /api/m/board/files/{itemId}`: streams the file, first fetching it from a holder if the desktop doesn't have it. It supports `Range`. Responses: `404` (unknown item), `503` (no online holder).
- The SSE stream gains a `board` event, sent on every board change, and the page re-fetches `GET /api/m/board`.
