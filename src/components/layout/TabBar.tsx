import type { TabId } from '../../stores/appStore';

const tabs: { id: TabId; label: string }[] = [
  { id: 'chat', label: 'Chat' },
  { id: 'tool-calls', label: 'Tool Calls' },
  { id: 'file-changes', label: 'File Changes' },
  { id: 'history', label: 'History' },
  { id: 'logs', label: 'Logs' },
];

export function TabBar(props: { activeTab: TabId; onChange: (tab: TabId) => void }) {
  return (
    <div className="lm-tabbar">
      {tabs.map((tab) => (
        <button
          key={tab.id}
          className={props.activeTab === tab.id ? 'lm-tab active' : 'lm-tab'}
          onClick={() => props.onChange(tab.id)}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
