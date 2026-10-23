import { Clipboard, FolderKanban, FolderUp, History, Pin, Radar, Smartphone, Upload, X, type LucideIcon } from "lucide-react";

import { ICON_SIZE, ICON_STROKE, type IconName } from "../../constants/icons";

const ICONS: Record<IconName, LucideIcon> = {
  nearby: Radar,
  sendFiles: Upload,
  sendFolder: FolderUp,
  clipboard: Clipboard,
  history: History,
  phone: Smartphone,
  pin: Pin,
  board: FolderKanban,
  close: X,
};

export interface IconProps {
  name: IconName;
  size?: number;
}

export function Icon({ name, size = ICON_SIZE }: IconProps) {
  const Glyph = ICONS[name];
  return <Glyph size={size} strokeWidth={ICON_STROKE} aria-hidden focusable={false} />;
}
