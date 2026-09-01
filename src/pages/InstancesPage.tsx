import { useState, useEffect } from "react";
import type { Account, TraeInstance } from "../types";
import * as api from "../api";

export default function InstancesPage() {
  const [instances, setInstances] = useState<TraeInstance[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [name, setName] = useState("");
  const [selectedAccount, setSelectedAccount] = useState("");
  const [error, setError] = useState("");
  const [launching, setLaunching] = useState<string | null>(null);
  const [runningMap, setRunningMap] = useState<Record<string, boolean>>({});

  const load = async () => {
    const [insts, accs] = await Promise.all([
      api.getInstances(),
      api.getAccounts(),
    ]);
    setInstances(insts);
    setAccounts(accs);
    if (accs.length > 0 && !selectedAccount) {
      setSelectedAccount(accs[0].id);
    }
  };

  useEffect(() => {
    load();
  }, []);

  useEffect(() => {
    const check = async () => {
      const map: Record<string, boolean> = {};
      for (const inst of instances) {
        try {
          map[inst.id] = await api.checkInstanceRunning(inst.id);
        } catch {
          map[inst.id] = false;
        }
      }
      setRunningMap(map);
    };
    check();
  }, [instances]);

  const handleCreate = async () => {
    if (!name.trim()) {
      setError("请输入实例名称");
      return;
    }
    if (!selectedAccount) {
      setError("请选择关联账号");
      return;
    }
    try {
      const acc = accounts.find((a) => a.id === selectedAccount);
      await api.createInstance(name.trim() || acc?.name || "instance", selectedAccount);
      setName("");
      setShowForm(false);
      setError("");
      await load();
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleLaunch = async (id: string) => {
    setLaunching(id);
    setError("");
    try {
      await api.launchInstance(id);
      setRunningMap((prev) => ({ ...prev, [id]: true }));
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setLaunching(null);
    }
  };

  const handleStop = async (id: string) => {
    try {
      await api.stopInstance(id);
      setRunningMap((prev) => ({ ...prev, [id]: false }));
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleRemove = async (id: string) => {
    try {
      await api.removeInstance(id);
      await load();
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const accountName = (id: string) =>
    accounts.find((a) => a.id === id)?.name ?? "未知";

  return (
    <div className="p-8 max-w-4xl">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-xl font-semibold text-white">实例管理</h2>
          <p className="text-sm text-gray-500 mt-1">
            每个实例拥有独立的 user-data-dir 和调试端口
          </p>
        </div>
        <button className="btn-primary" onClick={() => setShowForm(!showForm)}>
          {showForm ? "取消" : "创建实例"}
        </button>
      </div>

      {error && (
        <div className="mb-4 px-4 py-2.5 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
          {error}
        </div>
      )}

      {showForm && (
        <div className="card mb-6 space-y-3">
          <div>
            <label className="text-xs text-gray-400 block mb-1">
              关联账号
            </label>
            <select
              className="input"
              value={selectedAccount}
              onChange={(e) => setSelectedAccount(e.target.value)}
            >
              {accounts.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.name}
                </option>
              ))}
            </select>
          </div>
          <div>
            <label className="text-xs text-gray-400 block mb-1">
              实例名称（留空则使用账号名）
            </label>
            <input
              className="input"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="如：工作实例"
            />
          </div>
          <button className="btn-primary" onClick={handleCreate}>
            确认创建
          </button>
        </div>
      )}

      {instances.length === 0 ? (
        <div className="card text-center py-12 text-gray-500">
          <p className="text-sm">还没有创建任何实例</p>
          <p className="text-xs text-gray-600 mt-1">
            先在账号管理中添加账号，然后创建实例
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {instances.map((inst) => {
            const isRunning = runningMap[inst.id];
            const isLaunching = launching === inst.id;
            return (
              <div key={inst.id} className="card-hover">
                <div className="flex items-start justify-between">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-white text-sm">
                        {inst.name}
                      </span>
                      <span className="badge-neutral font-mono">
                        :{inst.debug_port}
                      </span>
                      {isRunning ? (
                        <span className="badge-success">运行中</span>
                      ) : (
                        <span className="badge-neutral">已停止</span>
                      )}
                    </div>
                    <div className="mt-2 space-y-0.5">
                      <p className="text-xs text-gray-500">
                        账号: {accountName(inst.account_id)}
                      </p>
                      <p className="text-xs text-gray-600 font-mono truncate">
                        {inst.data_dir}
                      </p>
                    </div>
                  </div>
                  <div className="flex items-center gap-2 shrink-0">
                    {isRunning ? (
                      <button
                        className="btn-danger text-xs"
                        onClick={() => handleStop(inst.id)}
                      >
                        停止
                      </button>
                    ) : (
                      <button
                        className="btn-primary text-xs"
                        onClick={() => handleLaunch(inst.id)}
                        disabled={isLaunching}
                      >
                        {isLaunching ? "启动中..." : "启动"}
                      </button>
                    )}
                    <button
                      className="btn-ghost text-xs"
                      onClick={() => handleRemove(inst.id)}
                    >
                      删除
                    </button>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}
