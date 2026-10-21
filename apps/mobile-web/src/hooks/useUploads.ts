import { useCallback, useEffect, useRef, useState } from "react";

import { UPLOAD_PATH } from "../constants/api";
import { api } from "../lib/api";
import type { UploadEntry } from "../types/api";

export interface UploadOptions {
  path?: string;
  onUploaded?: () => void;
}

export function useUploads({ path = UPLOAD_PATH, onUploaded }: UploadOptions = {}) {
  const [entries, setEntries] = useState<UploadEntry[]>([]);
  const nextId = useRef(0);
  const aborts = useRef(new Map<number, () => void>());
  const files = useRef(new Map<number, File>());

  const patch = useCallback((id: number, change: Partial<UploadEntry>) => {
    setEntries((all) => all.map((entry) => (entry.id === id ? { ...entry, ...change } : entry)));
  }, []);

  const start = useCallback((id: number, file: File) => {
    patch(id, { status: "uploading", progress: 0 });
    const handle = api.upload(file, (progress) => patch(id, { progress }), path);
    aborts.current.set(id, handle.abort);
    handle.promise
      .then(() => {
        patch(id, { status: "done", progress: 1 });
        onUploaded?.();
      })
      .catch(() => patch(id, { status: "failed" }))
      .finally(() => aborts.current.delete(id));
  }, [patch, path, onUploaded]);

  const add = useCallback((picked: File[]) => {
    const created = picked.map((file) => ({ id: nextId.current++, file, status: "uploading" as const, progress: 0 }));
    created.forEach((entry) => files.current.set(entry.id, entry.file));
    setEntries((all) => [...created, ...all]);
    created.forEach((entry) => start(entry.id, entry.file));
  }, [start]);

  const retry = useCallback((id: number) => {
    const file = files.current.get(id);
    if (file) start(id, file);
  }, [start]);

  useEffect(() => {
    const running = aborts.current;
    return () => running.forEach((abort) => abort());
  }, []);

  return { entries, add, retry };
}
