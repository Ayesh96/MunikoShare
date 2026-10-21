import {
  BOARD_FILES_PATH, BOARD_PATH, BOARD_TEXT_PATH, FILE_NAME_HEADER, HTTP_NO_CONTENT, HTTP_UNAUTHORIZED, INBOX_PATH, SESSION_PATH, TEXT_PATH, UPLOAD_PATH,
} from "../constants/api";
import type { BoardResponse, InboxItem, SessionInfo, SharedText } from "../types/api";

export class ApiError extends Error {
  constructor(readonly status: number) {
    super(`HTTP ${status}`);
  }
}

export interface UploadHandle {
  promise: Promise<void>;
  abort: () => void;
}

/** Typed client for /api/m; every 401 goes through one handler. */
export class ApiClient {
  private onUnauthorized: () => void = () => {};

  setUnauthorizedHandler(handler: () => void): void {
    this.onUnauthorized = handler;
  }

  private fail(status: number, notify: boolean): never {
    if (status === HTTP_UNAUTHORIZED && notify) this.onUnauthorized();
    throw new ApiError(status);
  }

  private async request(path: string, init: RequestInit = {}, notify = true): Promise<Response> {
    const response = await fetch(path, { credentials: "same-origin", ...init });
    if (!response.ok) this.fail(response.status, notify);
    return response;
  }

  private async json<T>(path: string, init?: RequestInit, notify = true): Promise<T> {
    return (await this.request(path, init, notify)).json() as Promise<T>;
  }

  /** Session probes never notify: a 401 there just means "show the scan screen". */
  createSession(token: string): Promise<SessionInfo> {
    const init = { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ token }) };
    return this.json<SessionInfo>(SESSION_PATH, init, false);
  }

  getSession(): Promise<SessionInfo> {
    return this.json<SessionInfo>(SESSION_PATH, undefined, false);
  }

  async deleteSession(): Promise<void> {
    await this.request(SESSION_PATH, { method: "DELETE" }, false);
  }

  listInbox(): Promise<InboxItem[]> {
    return this.json<InboxItem[]>(INBOX_PATH);
  }

  inboxUrl(id: string): string {
    return `${INBOX_PATH}/${encodeURIComponent(id)}`;
  }

  async sendText(text: string): Promise<void> {
    await this.postText(TEXT_PATH, text);
  }

  async postBoardText(text: string): Promise<void> {
    await this.postText(BOARD_TEXT_PATH, text);
  }

  getBoard(): Promise<BoardResponse> {
    return this.json<BoardResponse>(BOARD_PATH);
  }

  boardFileUrl(id: string): string {
    return `${BOARD_FILES_PATH}/${encodeURIComponent(id)}`;
  }

  private async postText(path: string, text: string): Promise<void> {
    const init = { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ text }) };
    await this.request(path, init);
  }

  async getText(): Promise<SharedText | null> {
    const response = await this.request(TEXT_PATH);
    return response.status === HTTP_NO_CONTENT ? null : ((await response.json()) as SharedText);
  }

  /** XHR instead of fetch because only XHR reports upload progress. */
  upload(file: File, onProgress: (fraction: number) => void, path: string = UPLOAD_PATH): UploadHandle {
    const xhr = new XMLHttpRequest();
    const promise = new Promise<void>((resolve, reject) => {
      xhr.open("POST", path);
      xhr.setRequestHeader(FILE_NAME_HEADER, encodeURIComponent(file.name));
      xhr.upload.onprogress = (event) => {
        if (event.lengthComputable) onProgress(event.loaded / event.total);
      };
      xhr.onload = () => {
        if (xhr.status >= 200 && xhr.status < 300) return resolve();
        if (xhr.status === HTTP_UNAUTHORIZED) this.onUnauthorized();
        reject(new ApiError(xhr.status));
      };
      xhr.onerror = () => reject(new ApiError(0));
      xhr.onabort = () => reject(new ApiError(0));
      xhr.send(file);
    });
    return { promise, abort: () => xhr.abort() };
  }
}

export const api = new ApiClient();
