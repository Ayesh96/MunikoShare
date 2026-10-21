import { EmptyState } from "./components/ui";
import { messages } from "./messages/app";
import { MainScreen } from "./features/MainScreen";
import { ScanScreen } from "./features/ScanScreen";
import { useSession } from "./hooks/useSession";

export function App() {
  const { state, disconnect, lose } = useSession();
  if (state.status === "loading") return <EmptyState message={messages.loading} />;
  if (state.status === "scan") return <ScanScreen />;
  return <MainScreen deviceName={state.deviceName} onDisconnect={disconnect} onRevoked={lose} />;
}
