import { formatBytes } from "../lib/format";
import { ACCEPT_LABEL, DECLINE_LABEL, OFFER_TITLE, offerPrompt, offerTotal } from "../messages/ui";
import type { IncomingOffer } from "../types/transfer";
import { List, ListItem, Modal } from "./ui";

export interface OfferModalProps {
  offer: IncomingOffer;
  senderName: string;
  onRespond: (accept: boolean) => void;
}

export function OfferModal({ offer, senderName, onRespond }: OfferModalProps) {
  return (
    <Modal
      open
      title={OFFER_TITLE}
      onClose={() => onRespond(false)}
      actions={[
        { label: DECLINE_LABEL, onClick: () => onRespond(false) },
        { label: ACCEPT_LABEL, onClick: () => onRespond(true), variant: "primary" },
      ]}
    >
      <p>{offerPrompt(senderName)}</p>
      <List>
        {offer.files.map((file, index) => (
          <ListItem key={`${index}-${file.name}`} title={file.name} subtitle={formatBytes(file.size)} />
        ))}
      </List>
      <p>{offerTotal(formatBytes(offer.total_size))}</p>
    </Modal>
  );
}
