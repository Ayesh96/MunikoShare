import { TOKEN_HASH_KEY } from "../constants/api";
import type { SessionInfo } from "../types/api";
import { api } from "./api";

let pending: Promise<SessionInfo | null> | null = null;

function takeTokenFromHash(): string | null {
  const token = new URLSearchParams(location.hash.slice(1)).get(TOKEN_HASH_KEY);
  if (token) history.replaceState(null, "", location.pathname + location.search);
  return token;
}

async function establish(): Promise<SessionInfo | null> {
  const token = takeTokenFromHash();
  try {
    return await (token ? api.createSession(token) : api.getSession());
  } catch {
    return null;
  }
}

/** Memoized because the token is single-use and effects can run twice in dev. */
export function bootstrapSession(): Promise<SessionInfo | null> {
  pending ??= establish();
  return pending;
}

export function resetBootstrap(): void {
  pending = null;
}
