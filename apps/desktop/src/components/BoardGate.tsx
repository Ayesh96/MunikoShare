import type { ReactNode } from "react";

import { BOARD_SETUP_ICON } from "../constants/icons";
import {
  BOARD_CONFLICT_NOTICE,
  BOARD_CONFLICT_REVIEW,
  BOARD_OPEN_SETTINGS_LABEL,
  BOARD_PENDING_HINT,
  BOARD_PENDING_TITLE,
} from "../messages/board";
import type { BoardState } from "../types/board";
import { Banner, Card, EmptyState } from "./ui";

export interface BoardGateProps {
  state: BoardState | null;
  onOpenSettings: () => void;
  children: ReactNode;
}

/** Shows the board-backed tabs once a board is ready; board management itself lives in Settings. */
export function BoardGate({ state, onOpenSettings, children }: BoardGateProps) {
  if (!state) return null;
  return (
    <>
      {state.conflicts.length > 0 && (
        <Banner tone="info" message={BOARD_CONFLICT_NOTICE} actionLabel={BOARD_CONFLICT_REVIEW} onAction={onOpenSettings} />
      )}
      {state.board ? (
        children
      ) : (
        <Card>
          <EmptyState
            icon={BOARD_SETUP_ICON}
            title={BOARD_PENDING_TITLE}
            hint={BOARD_PENDING_HINT}
            actionLabel={BOARD_OPEN_SETTINGS_LABEL}
            onAction={onOpenSettings}
          />
        </Card>
      )}
    </>
  );
}
