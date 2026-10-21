import {
  BOARD_KEEPER_HINT,
  BOARD_KEEPER_LABEL,
  BOARD_LEAVE_LABEL,
  BOARD_MODE_OPEN,
  BOARD_MODE_PROTECTED,
} from "../messages/board";
import type { Board } from "../types/board";
import styles from "./BoardHeader.module.css";
import { Badge, Button, Card, Switch } from "./ui";

export interface BoardHeaderProps {
  board: Board;
  keeper: boolean;
  onKeeperChange: (keeper: boolean) => void;
  onLeave: () => void;
}

export function BoardHeader({ board, keeper, onKeeperChange, onLeave }: BoardHeaderProps) {
  return (
    <Card title={board.name}>
      <div className={styles.row}>
        <Badge tone={board.mode === "open" ? "neutral" : "success"} label={board.mode === "open" ? BOARD_MODE_OPEN : BOARD_MODE_PROTECTED} />
        <span className={styles.spacer} />
        <Button size="sm" variant="secondary" label={BOARD_LEAVE_LABEL} onClick={onLeave} />
      </div>
      <Switch checked={keeper} onChange={onKeeperChange} label={BOARD_KEEPER_LABEL} />
      <div className={styles.hint}>{BOARD_KEEPER_HINT}</div>
    </Card>
  );
}
