import styles from "./Tabs.module.css";

export interface TabOption<T extends string> {
  id: T;
  label: string;
}

export interface TabsProps<T extends string> {
  tabs: TabOption<T>[];
  active: T;
  onChange: (id: T) => void;
}

export function Tabs<T extends string>({ tabs, active, onChange }: TabsProps<T>) {
  return (
    <div className={styles.tabs} role="tablist">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          role="tab"
          aria-selected={tab.id === active}
          className={`${styles.tab} ${tab.id === active ? styles.active : ""}`}
          onClick={() => onChange(tab.id)}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
