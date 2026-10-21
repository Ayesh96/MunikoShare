import { useState } from "react";

import { PHONE_POLL_INTERVAL_MS } from "../constants/events";
import { useInterval } from "../hooks/useInterval";
import type { Phones } from "../hooks/usePhones";
import { PHONE_CONNECT_LABEL, PHONE_TITLE } from "../messages/phone";
import { PHONE_ACCESS_OFF_HINT, PHONE_ACCESS_OFF_ICON, PHONE_ACCESS_OFF_TITLE } from "../messages/settings";
import type { PhoneSession } from "../types/phone";
import { ConnectPhoneModal } from "./ConnectPhoneModal";
import { PhoneList } from "./PhoneList";
import { SendPhoneTextModal } from "./SendPhoneTextModal";
import { Button, Card, EmptyState } from "./ui";

/** Mounted only while its tab is visible, so the 10 s refresh stops when the tab changes. */
export function PhoneTab({ phones }: { phones: Phones }) {
  const [connecting, setConnecting] = useState(false);
  const [textTarget, setTextTarget] = useState<PhoneSession | null>(null);
  useInterval(phones.refresh, PHONE_POLL_INTERVAL_MS);

  return (
    <>
      <Card title={PHONE_TITLE}>
        {phones.access ? (
          <Button label={PHONE_CONNECT_LABEL} onClick={() => setConnecting(true)} />
        ) : (
          <EmptyState icon={PHONE_ACCESS_OFF_ICON} title={PHONE_ACCESS_OFF_TITLE} hint={PHONE_ACCESS_OFF_HINT} />
        )}
      </Card>
      {phones.access && (
        <PhoneList sessions={phones.sessions} onSendFiles={phones.sendFiles} onSendText={setTextTarget} onDisconnect={phones.disconnect} />
      )}
      {connecting && <ConnectPhoneModal onClose={() => setConnecting(false)} />}
      {textTarget && (
        <SendPhoneTextModal onSend={(text) => phones.sendText(textTarget, text)} onClose={() => setTextTarget(null)} />
      )}
    </>
  );
}
