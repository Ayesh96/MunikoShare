import { BOARD_SHARE_FILES_ICON } from "../constants/icons";
import { BOARD_ADD_FILES_LABEL, BOARD_DROP_HINT, BOARD_SHARE_TITLE } from "../messages/board";
import styles from "./ShareFilesCard.module.css";
import { Button, Card } from "./ui";

export function ShareFilesCard({ onAddFiles }: { onAddFiles: () => void }) {
  return (
    <Card title={BOARD_SHARE_TITLE}>
      <div className={styles.row}>
        <Button icon={BOARD_SHARE_FILES_ICON} label={BOARD_ADD_FILES_LABEL} onClick={onAddFiles} />
        <span className={styles.hint}>{BOARD_DROP_HINT}</span>
      </div>
    </Card>
  );
}
