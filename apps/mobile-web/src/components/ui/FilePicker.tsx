import { useRef } from "react";

import { Button } from "./Button";

export interface FilePickerProps {
  label: string;
  onPick: (files: File[]) => void;
}

/** Hidden multi-file input opened by a large button. */
export function FilePicker({ label, onPick }: FilePickerProps) {
  const input = useRef<HTMLInputElement>(null);
  return (
    <>
      <input
        ref={input}
        type="file"
        multiple
        hidden
        onChange={(event) => {
          onPick(Array.from(event.target.files ?? []));
          event.target.value = "";
        }}
      />
      <Button label={label} fullWidth onClick={() => input.current?.click()} />
    </>
  );
}
