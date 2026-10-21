import { useState } from "react";

import { Button, Tabs } from "../components/ui";
import { useBoard } from "../hooks/useBoard";
import { useInbox } from "../hooks/useInbox";
import { useLiveUpdates } from "../hooks/useLiveUpdates";
import { useReceivedText } from "../hooks/useReceivedText";
import { messages } from "../messages/app";
import { BoardScreen } from "./BoardScreen";
import { Inbox } from "./Inbox";
import { ReceivedText } from "./ReceivedText";
import { SendFiles } from "./SendFiles";
import { SendText } from "./SendText";
import styles from "./Screen.module.css";

export interface MainScreenProps {
  deviceName: string;
  onDisconnect: () => void;
  onRevoked: () => void;
}

type TabId = "transfer" | "board";
const TABS = [
  { id: "transfer", label: messages.tabs.transfer },
  { id: "board", label: messages.tabs.board },
] satisfies { id: TabId; label: string }[];

export function MainScreen({ deviceName, onDisconnect, onRevoked }: MainScreenProps) {
  const [tab, setTab] = useState<TabId>("transfer");
  const inbox = useInbox();
  const text = useReceivedText();
  const board = useBoard();
  useLiveUpdates({ onInbox: inbox.refresh, onText: text.refresh, onBoard: board.refresh, onRevoked });

  return (
    <main className={styles.page}>
      <header className={styles.header}>
        <h1 className={styles.heading}>{messages.header.connectedTo(deviceName)}</h1>
        <Button label={messages.header.disconnect} size="sm" variant="secondary" onClick={onDisconnect} />
      </header>
      <Tabs tabs={TABS} active={tab} onChange={setTab} />
      {tab === "transfer" ? (
        <>
          <SendFiles />
          <Inbox items={inbox.items} />
          <SendText />
          <ReceivedText shared={text.shared} />
        </>
      ) : (
        <BoardScreen data={board.data} onChanged={board.refresh} />
      )}
    </main>
  );
}
