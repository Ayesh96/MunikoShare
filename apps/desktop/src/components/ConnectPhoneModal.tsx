import { QR_SIZE_PX } from "../constants/board";
import { useCountdown } from "../hooks/useCountdown";
import { usePhoneInvite } from "../hooks/usePhoneInvite";
import { formatCountdown } from "../lib/phone";
import {
  PHONE_CERT_NOTE,
  PHONE_CLOSE_LABEL,
  PHONE_CONNECT_STEPS,
  PHONE_CONNECT_TITLE,
  PHONE_EXPIRED,
  PHONE_NEW_CODE_LABEL,
  PHONE_OTHER_LINKS,
  phoneExpiresIn,
} from "../messages/phone";
import styles from "./ConnectPhoneModal.module.css";
import { Banner, Modal, QrCode } from "./ui";

export function ConnectPhoneModal({ onClose }: { onClose: () => void }) {
  const { invite, error, renew } = usePhoneInvite();
  const secondsLeft = useCountdown(invite?.expiresAt ?? null);
  const expired = invite !== null && secondsLeft === 0;
  const [first, ...others] = invite?.urls ?? [];

  return (
    <Modal
      open
      title={PHONE_CONNECT_TITLE}
      onClose={onClose}
      actions={[
        ...(expired ? [{ label: PHONE_NEW_CODE_LABEL, onClick: renew, variant: "primary" as const }] : []),
        { label: PHONE_CLOSE_LABEL, onClick: onClose },
      ]}
    >
      <div className={styles.body}>
        {error && <Banner message={error} tone="danger" />}
        {first && (
          <>
            <span className={styles.muted}>{PHONE_CONNECT_STEPS}</span>
            <QrCode value={first} size={QR_SIZE_PX} />
            <span className={styles.url}>{first}</span>
            <span className={styles.muted}>{expired ? PHONE_EXPIRED : phoneExpiresIn(formatCountdown(secondsLeft))}</span>
            {others.length > 0 && (
              <>
                <span className={styles.muted}>{PHONE_OTHER_LINKS}</span>
                {others.map((url) => (
                  <span key={url} className={styles.url}>{url}</span>
                ))}
              </>
            )}
            <span className={styles.muted}>{PHONE_CERT_NOTE}</span>
          </>
        )}
      </div>
    </Modal>
  );
}
