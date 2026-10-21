import { useEffect, useRef } from "react";

import { onFilesDropped } from "../lib/tauriApi";

/** Calls `onDrop` with the paths of files dropped on the window; unlistens on unmount. */
export function useFileDrop(onDrop: (paths: string[]) => void): void {
  const dropRef = useRef(onDrop);
  dropRef.current = onDrop;

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | null = null;
    onFilesDropped((paths) => dropRef.current(paths)).then((fn) => {
      if (cancelled) fn();
      else unlisten = fn;
    });
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);
}
