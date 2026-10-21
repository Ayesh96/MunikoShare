import { useState } from "react";

import { Banner, Button, Card, FilePicker, TextArea } from "../components/ui";
import { BOARD_FILES_PATH } from "../constants/api";
import { useUploads } from "../hooks/useUploads";
import { api } from "../lib/api";
import { messages } from "../messages/app";
import { UploadList } from "./UploadList";

export function BoardComposer({ onPosted }: { onPosted: () => void }) {
  const [text, setText] = useState("");
  const [failed, setFailed] = useState(false);
  const { entries, add, retry } = useUploads({ path: BOARD_FILES_PATH, onUploaded: onPosted });

  const post = async () => {
    try {
      await api.postBoardText(text);
      setText("");
      setFailed(false);
      onPosted();
    } catch {
      setFailed(true);
    }
  };

  return (
    <Card>
      <TextArea value={text} onChange={setText} placeholder={messages.board.composePlaceholder} />
      <Button label={messages.board.post} fullWidth disabled={text.trim() === ""} onClick={post} />
      {failed && <Banner message={messages.board.postFailed} tone="danger" />}
      <FilePicker label={messages.board.addFiles} onPick={add} />
      {entries.length > 0 && <UploadList entries={entries} onRetry={retry} />}
    </Card>
  );
}
