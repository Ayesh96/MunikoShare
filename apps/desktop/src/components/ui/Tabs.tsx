import type { IconName } from "../../constants/icons";
import { Icon } from "./Icon";
import styles from "./Tabs.module.css";

export interface TabItem {
  id: string;
  label: string;
  icon?: IconName;
}

export interface TabsProps {
  items: TabItem[];
  activeId: string;
  onChange: (id: string) => void;
}

export function Tabs({ items, activeId, onChange }: TabsProps) {
  return (
    <div className={styles.tabs} role="tablist">
      {items.map((item) => (
        <button
          type="button"
          key={item.id}
          role="tab"
          aria-selected={item.id === activeId}
          className={`${styles.tab} ${item.id === activeId ? styles.active : ""}`}
          onClick={() => onChange(item.id)}
        >
          {item.icon && <Icon name={item.icon} />}
          <span className={styles.label}>{item.label}</span>
        </button>
      ))}
    </div>
  );
}
