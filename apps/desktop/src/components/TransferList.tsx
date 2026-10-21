import { formatBytes } from "../lib/format";
import {
  DIRECTION_INCOMING,
  DIRECTION_OUTGOING,
  TRANSFERS_TITLE,
  transferLabel,
  waitingForAccept,
} from "../messages/ui";
import type { ActiveTransfer } from "../types/transfer";
import { Card, List, ListItem, ProgressBar } from "./ui";

export interface TransferListProps {
  transfers: ActiveTransfer[];
  waitingFor: string[];
  nameOf: (deviceId: string) => string;
}

export function TransferList({ transfers, waitingFor, nameOf }: TransferListProps) {
  if (transfers.length === 0 && waitingFor.length === 0) return null;
  return (
    <Card title={TRANSFERS_TITLE}>
      <List>
        {waitingFor.map((deviceId) => (
          <ListItem key={`waiting-${deviceId}`} title={waitingForAccept(nameOf(deviceId))} />
        ))}
      </List>
      {transfers.map((transfer) => {
        const fraction = transfer.bytes_total > 0 ? transfer.bytes_done / transfer.bytes_total : 0;
        const direction = transfer.direction === "incoming" ? DIRECTION_INCOMING : DIRECTION_OUTGOING;
        const label = transferLabel(
          direction,
          nameOf(transfer.device_id),
          Math.round(fraction * 100),
          formatBytes(transfer.bytes_done),
          formatBytes(transfer.bytes_total),
        );
        return <ProgressBar key={transfer.transfer_id} value={fraction} label={label} />;
      })}
    </Card>
  );
}
