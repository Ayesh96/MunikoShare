import { BOARD_SETUP_ICON } from "../constants/icons";
import { BOARD_CREATE_LABEL, BOARD_DISCOVERED_TITLE, BOARD_JOIN_LABEL, BOARD_MODE_OPEN, BOARD_MODE_PROTECTED, BOARD_SETUP_HINT, BOARD_SETUP_TITLE, boardDiscoveredRow } from "../messages/board";
import type { ObservedBoard } from "../types/board";
import { Badge, Button, Card, EmptyState, List, ListItem } from "./ui";

export interface BoardSetupProps {
  discovered: ObservedBoard[];
  onCreate: () => void;
  onJoin: (board: ObservedBoard) => void;
}

/** Shown while this device has no board. */
export function BoardSetup({ discovered, onCreate, onJoin }: BoardSetupProps) {
  return (
    <>
      <Card>
        <EmptyState icon={BOARD_SETUP_ICON} title={BOARD_SETUP_TITLE} hint={BOARD_SETUP_HINT} />
        <Button label={BOARD_CREATE_LABEL} onClick={onCreate} />
      </Card>
      {discovered.length > 0 && (
        <Card title={BOARD_DISCOVERED_TITLE}>
          <List>
            {discovered.map((board) => (
              <ListItem
                key={board.info.id}
                title={board.info.name}
                subtitle={boardDiscoveredRow(board.peerName, board.info.memberCount)}
                leading={<Badge tone={board.info.mode === "open" ? "neutral" : "success"} label={board.info.mode === "open" ? BOARD_MODE_OPEN : BOARD_MODE_PROTECTED} />}
                trailing={<Button size="sm" label={BOARD_JOIN_LABEL} onClick={() => onJoin(board)} />}
              />
            ))}
          </List>
        </Card>
      )}
    </>
  );
}
