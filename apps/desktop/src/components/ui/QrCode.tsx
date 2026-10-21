import { useEffect, useState } from "react";

import { qrDataUrl } from "../../lib/qr";
import { QR_ALT } from "../../messages/phone";

export interface QrCodeProps {
  value: string;
  size: number;
}

export function QrCode({ value, size }: QrCodeProps) {
  const [url, setUrl] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    qrDataUrl(value, size)
      .then((result) => !cancelled && setUrl(result))
      .catch(() => !cancelled && setUrl(null));
    return () => {
      cancelled = true;
    };
  }, [value, size]);

  if (!url) return <div style={{ width: size, height: size }} aria-hidden />;
  return <img src={url} width={size} height={size} alt={QR_ALT} />;
}
