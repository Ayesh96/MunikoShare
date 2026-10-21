import { useState } from "react";

import { BOARD_ADD_FILES_LABEL, BOARD_COMPOSER_PLACEHOLDER, BOARD_POST_LABEL } from "../messages/board";
import styles from "./BoardComposer.module.css";
import { Button, Card, TextArea } from "./ui";

export interface BoardComposerProps {
  onPost: (text: string) => Promise<boolean>;
  onAddFiles: () => void;
}

export function BoardComposer({ onPost, onAddFiles }: BoardComposerProps) {
  const [text, setText] = useState("");

  async function post() {
    if (await onPost(text)) setText("");
  }

  return (
    <Card>
      <TextArea value={text} onChange={setText} placeholder={BOARD_COMPOSER_PLACEHOLDER} rows={3} />
      <div className={styles.buttons}>
        <Button size="sm" label={BOARD_POST_LABEL} disabled={text.trim() === ""} onClick={post} />
        <Button size="sm" variant="secondary" label={BOARD_ADD_FILES_LABEL} onClick={onAddFiles} />
      </div>
    </Card>
  );
}
