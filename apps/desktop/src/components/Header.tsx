import { ADD_BY_IP_LABEL, APP_NAME, deviceEndpoint, LOADING_DEVICE } from "../messages/ui";
import type { DeviceInfo } from "../types/device";
import { Button, Card, List, ListItem } from "./ui";

export interface HeaderProps {
  device: DeviceInfo | null;
  addresses: string[];
  onAddDevice: () => void;
}

export function Header({ device, addresses, onAddDevice }: HeaderProps) {
  return (
    <Card>
      <List>
        <ListItem
          title={APP_NAME}
          trailing={<Button size="sm" variant="secondary" label={ADD_BY_IP_LABEL} onClick={onAddDevice} />}
        />
        <ListItem
          title={device?.name ?? LOADING_DEVICE}
          subtitle={device ? deviceEndpoint(addresses, device.port) : undefined}
        />
      </List>
    </Card>
  );
}
