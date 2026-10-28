import { useFileDrop } from "../hooks/useFileDrop";
import { DeviceList, type DeviceListProps } from "./DeviceList";

export interface DevicesTabProps extends DeviceListProps {
  onDropPaths: (paths: string[]) => void;
}

/** Mounted only while its tab is visible, so drops here open the send picker instead of posting to the board. */
export function DevicesTab({ onDropPaths, ...listProps }: DevicesTabProps) {
  useFileDrop((paths) => paths.length > 0 && onDropPaths(paths));
  return <DeviceList {...listProps} />;
}
