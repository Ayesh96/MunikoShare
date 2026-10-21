import { SERVICE_WORKER_FILE } from "../constants/api";

/** Production only, so dev reloads never serve a stale shell. */
export function registerServiceWorker(): void {
  if (!import.meta.env.PROD || !("serviceWorker" in navigator)) return;
  window.addEventListener("load", () => {
    navigator.serviceWorker.register(`${import.meta.env.BASE_URL}${SERVICE_WORKER_FILE}`).catch(() => {});
  });
}
