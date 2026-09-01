import { useState, useEffect } from "react";
import AccountsPage from "./pages/AccountsPage";
import InstancesPage from "./pages/InstancesPage";
import CheckinPage from "./pages/CheckinPage";
import SchedulerPage from "./pages/SchedulerPage";
import { findTraePath } from "./api";

type Tab = "checkin" | "accounts" | "instances" | "scheduler";

const navItems: { id: Tab; label: string }[] = [
  { id: "checkin", label: "签到中心" },
  { id: "accounts", label: "账号管理" },
  { id: "instances", label: "实例管理" },
  { id: "scheduler", label: "定时设置" },
];

function App() {
  const [tab, setTab] = useState<Tab>("checkin");
  const [traePath, setTraePath] = useState<string>("检测中...");

  useEffect(() => {
    findTraePath()
      .then(setTraePath)
      .catch(() => setTraePath("未找到"));
  }, []);

  return (
    <div className="flex h-screen">
      <aside className="w-56 bg-bg-secondary border-r border-white/5 flex flex-col shrink-0">
        <div className="p-5 pb-4">
          <h1 className="text-base font-bold tracking-tight text-white">
            TRAE 签到管理
          </h1>
          <p className="text-[11px] text-gray-500 mt-0.5">多账号自动签到工具</p>
        </div>

        <nav className="flex-1 px-3 space-y-0.5">
          {navItems.map((item) => (
            <button
              key={item.id}
              onClick={() => setTab(item.id)}
              className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-all duration-150 ${
                tab === item.id
                  ? "bg-white/10 text-white font-medium"
                  : "text-gray-400 hover:bg-white/5 hover:text-gray-200"
              }`}
            >
              {item.label}
            </button>
          ))}
        </nav>

        <div className="p-4 border-t border-white/5">
          <p className="text-[10px] text-gray-600 uppercase tracking-wider mb-1">
            TRAE 路径
          </p>
          <p
            className="text-[11px] text-gray-500 truncate font-mono"
            title={traePath}
          >
            {traePath}
          </p>
        </div>
      </aside>

      <main className="flex-1 overflow-y-auto">
        {tab === "checkin" && <CheckinPage />}
        {tab === "accounts" && <AccountsPage />}
        {tab === "instances" && <InstancesPage />}
        {tab === "scheduler" && <SchedulerPage />}
      </main>
    </div>
  );
}

export default App;
