import { useState } from "react";

import styles from "./App.module.css";
import { AddDeviceModal } from "./components/AddDeviceModal";
import { BoardTab } from "./components/BoardTab";
import { ClipboardTab } from "./components/ClipboardTab";
import { DeviceList } from "./components/DeviceList";
import { Header } from "./components/Header";
import { HistoryList } from "./components/HistoryList";
import { OfferModal } from "./components/OfferModal";
import { PairCodeModal } from "./components/PairCodeModal";
import { PairEnterModal } from "./components/PairEnterModal";
import { PhoneTab } from "./components/PhoneTab";
import { SendToModal } from "./components/SendToModal";
import { SettingsTab } from "./components/SettingsTab";
import { TransferList } from "./components/TransferList";
import { Banner, Tabs } from "./components/ui";
import { TAB_BOARD, TAB_CLIPBOARD, TAB_DEVICES, TAB_HISTORY, TAB_PHONE, TAB_SETTINGS } from "./constants/app";
import { useAddDevice } from "./hooks/useAddDevice";
import { useAutoAccept } from "./hooks/useAutoAccept";
import { useClipboardNotice } from "./hooks/useClipboardNotice";
import { useClipboardSync } from "./hooks/useClipboardSync";
import { useDeviceActions } from "./hooks/useDeviceActions";
import { useDeviceNames } from "./hooks/useDeviceNames";
import { useHistory } from "./hooks/useHistory";
import { useHotkeyNotice } from "./hooks/useHotkeyNotice";
import { useIncomingOffers } from "./hooks/useIncomingOffers";
import { useLaunchPaths } from "./hooks/useLaunchPaths";
import { useLocalAddresses } from "./hooks/useLocalAddresses";
import { useLocalDevice } from "./hooks/useLocalDevice";
import { useNotice } from "./hooks/useNotice";
import { usePairing } from "./hooks/usePairing";
import { usePendingSends } from "./hooks/usePendingSends";
import { usePeers } from "./hooks/usePeers";
import { usePhones } from "./hooks/usePhones";
import { useTransfers } from "./hooks/useTransfers";
import { useTrusted } from "./hooks/useTrusted";
import { TAB_BOARD_LABEL } from "./messages/board";
import { TAB_PHONE_LABEL } from "./messages/phone";
import { sendToStarted, TAB_SETTINGS_LABEL } from "./messages/settings";
import { TAB_CLIPBOARD_LABEL, TAB_DEVICES_LABEL, TAB_HISTORY_LABEL } from "./messages/ui";

const TABS = [
  { id: TAB_DEVICES, label: TAB_DEVICES_LABEL },
  { id: TAB_BOARD, label: TAB_BOARD_LABEL },
  { id: TAB_CLIPBOARD, label: TAB_CLIPBOARD_LABEL },
  { id: TAB_PHONE, label: TAB_PHONE_LABEL },
  { id: TAB_HISTORY, label: TAB_HISTORY_LABEL },
  { id: TAB_SETTINGS, label: TAB_SETTINGS_LABEL },
];

export function App() {
  const [tab, setTab] = useState(TAB_DEVICES);
  const device = useLocalDevice();
  const { peers, loading: peersLoading } = usePeers();
  const trusted = useTrusted();
  const { notice, show, clear } = useNotice();
  const phones = usePhones(show);
  const nameOf = useDeviceNames(peers, trusted.devices, phones.sessions);
  const history = useHistory();
  const addresses = useLocalAddresses();
  const transfers = useTransfers();
  const pendingSends = usePendingSends();
  const actions = useDeviceActions(show, pendingSends);
  const autoAccept = useAutoAccept(show);
  const clipboardSync = useClipboardSync(show);
  const addDevice = useAddDevice(show);
  const pairing = usePairing((message) => show(message, "danger"));
  const offers = useIncomingOffers((message) => show(message, "danger"));
  useClipboardNotice(nameOf, show);
  useHotkeyNotice(show);
  const launch = useLaunchPaths();
  const pairedPeers = peers.filter((peer) => trusted.ids.has(peer.id));

  return (
    <main className={styles.page}>
      <Header device={device} addresses={addresses} onAddDevice={addDevice.open} />
      {notice && <Banner message={notice.message} tone={notice.tone} onDismiss={clear} />}
      <TransferList transfers={transfers} waitingFor={pendingSends.waiting} nameOf={nameOf} />
      <Tabs items={TABS} activeId={tab} onChange={setTab} />
      {tab === TAB_DEVICES ? (
        <DeviceList
          peers={peers}
          loading={peersLoading}
          onAddDevice={addDevice.open}
          trusted={trusted.ids}
          autoAccept={autoAccept.enabled}
          clipboardSync={clipboardSync.enabled}
          onSendFiles={actions.sendFilesTo}
          onSendFolder={actions.sendFolderTo}
          onSendClipboard={actions.sendClipboardTo}
          onToggleAutoAccept={(peer, enabled) => autoAccept.toggle(peer.id, enabled)}
          onToggleClipboardSync={(peer, enabled) => clipboardSync.toggle(peer.id, enabled)}
          onPair={pairing.start}
          onUnpair={actions.unpairDevice}
        />
      ) : tab === TAB_BOARD ? (
        <BoardTab show={show} />
      ) : tab === TAB_PHONE ? (
        <PhoneTab phones={phones} />
      ) : tab === TAB_CLIPBOARD ? (
        <ClipboardTab
          pairedPeers={pairedPeers}
          nameOf={nameOf}
          show={show}
          onSendText={actions.sendTextTo}
        />
      ) : tab === TAB_SETTINGS ? (
        <SettingsTab phones={phones} show={show} />
      ) : (
        <HistoryList
          entries={history.entries}
          loading={history.loading}
          failed={history.failed}
          nameOf={nameOf}
          onClear={history.clear}
          onRetry={history.reload}
          onGoToDevices={() => setTab(TAB_DEVICES)}
        />
      )}
      {launch.paths.length > 0 && (
        <SendToModal
          paths={launch.paths}
          devices={pairedPeers}
          phones={phones.access ? phones.sessions : []}
          onSendDevice={(device) => {
            actions.sendPathsTo(device, launch.paths);
            show(sendToStarted(device.name), "info");
            launch.dismiss();
          }}
          onSendPhone={(phone) => {
            phones.sendPaths(phone, launch.paths);
            launch.dismiss();
          }}
          onClose={launch.dismiss}
        />
      )}
      {addDevice.isOpen && <AddDeviceModal onAdd={addDevice.add} onClose={addDevice.close} />}
      {pairing.target && (
        <PairEnterModal key={pairing.target.id} device={pairing.target} onConfirm={pairing.confirm} onClose={pairing.cancel} />
      )}
      {pairing.incoming && (
        <PairCodeModal device={pairing.incoming.device} code={pairing.incoming.code} onClose={pairing.dismissIncoming} />
      )}
      {offers.current && (
        <OfferModal offer={offers.current} senderName={nameOf(offers.current.from_device_id)} onRespond={offers.respond} />
      )}
    </main>
  );
}
