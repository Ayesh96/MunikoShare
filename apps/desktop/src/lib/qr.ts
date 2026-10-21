import QRCode from "qrcode";

import { QR_MARGIN_MODULES } from "../constants/board";
import { qr } from "../theme/tokens";

/** PNG data URL of a QR code; always dark on light so phone cameras can read it in dark mode too. */
export function qrDataUrl(value: string, size: number): Promise<string> {
  return QRCode.toDataURL(value, {
    width: size,
    margin: QR_MARGIN_MODULES,
    color: { dark: qr.dark, light: qr.light },
  });
}
