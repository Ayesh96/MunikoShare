import { useEffect, useRef } from "react";

import { pastedImage } from "../lib/image";

/** Calls `onImage` when an image is pasted anywhere in the window; text pastes are left alone. */
export function useImagePaste(onImage: (image: Blob) => void): void {
  const imageRef = useRef(onImage);
  imageRef.current = onImage;

  useEffect(() => {
    const onPaste = (event: ClipboardEvent) => {
      const image = pastedImage(event.clipboardData);
      if (!image) return;
      event.preventDefault();
      imageRef.current(image);
    };
    document.addEventListener("paste", onPaste);
    return () => document.removeEventListener("paste", onPaste);
  }, []);
}
