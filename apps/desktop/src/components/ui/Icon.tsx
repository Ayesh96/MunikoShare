import { ChevronDown, ChevronUp, Clipboard, Copy, Download, File, FolderKanban, FolderUp, History, Image, Pin, Radar, Send, Settings, Smartphone, Trash2, Type, Upload, X, type LucideIcon } from "lucide-react";

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
  settings: Settings,
  send: Send,
  copy: Copy,
  download: Download,
  trash: Trash2,
  expand: ChevronDown,
  collapse: ChevronUp,
  text: Type,
  file: File,
  image: Image,
};

export interface IconProps {
  name: IconName;
  size?: number;
}

export function Icon({ name, size = ICON_SIZE }: IconProps) {
  const Glyph = ICONS[name];
  return <Glyph size={size} strokeWidth={ICON_STROKE} aria-hidden focusable={false} />;
}
