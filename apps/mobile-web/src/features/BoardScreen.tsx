import { Card, EmptyState } from "../components/ui";
import { messages } from "../messages/app";
import type { BoardResponse } from "../types/api";
import styles from "./Screen.module.css";
import { BoardComposer } from "./BoardComposer";
import { BoardFeed } from "./BoardFeed";

export interface BoardScreenProps {
  data: BoardResponse | null;
  onChanged: () => void;
}

export function BoardScreen({ data, onChanged }: BoardScreenProps) {
  if (!data) return <EmptyState message={messages.board.loading} />;
  if (!data.board) return <Card><EmptyState message={messages.board.notReady} /></Card>;
  return (
    <>
      <h2 className={styles.heading}>{data.board.name}</h2>
      <BoardComposer onPosted={onChanged} />
      <BoardFeed items={data.items} />
    </>
  );
}
