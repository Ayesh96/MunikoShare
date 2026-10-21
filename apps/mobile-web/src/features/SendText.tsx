import { useState } from "react";

import { Banner, Button, Card, TextArea } from "../components/ui";
import { api } from "../lib/api";
import { messages } from "../messages/app";

export function SendText() {
  const [text, setText] = useState("");
  const [result, setResult] = useState<"sent" | "failed" | null>(null);

  const submit = async () => {
    try {
      await api.sendText(text);
      setText("");
      setResult("sent");
    } catch {
      setResult("failed");
    }
  };

  return (
    <Card title={messages.text.sendTitle}>
      <TextArea value={text} onChange={setText} placeholder={messages.text.placeholder} />
      <Button label={messages.text.send} fullWidth disabled={text.trim() === ""} onClick={submit} />
      {result && <Banner message={result === "sent" ? messages.text.sent : messages.text.sendFailed} tone={result === "sent" ? "success" : "danger"} />}
    </Card>
  );
}
