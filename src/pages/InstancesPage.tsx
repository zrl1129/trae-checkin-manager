import { useState, useEffect, useCallback } from "react";
import type { InstanceBrief, Account, SafeCleanItem } from "../types";
import { formatBytes, formatRelativeTime } from "../types";
import * as api from "../api";

export default function InstancesPage() {
  const [briefs, setBriefs] = useState<InstanceBrief[]>([]);
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [name, setName] = useState("");
  const [selectedAccount, setSelectedAccount] = useState("");
  const [error, setError] = useState("");
  const [launching, setLaunching] = useState<string | null>(null);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const [editNote, setEditNote] = useState("");
  const [cleanItems, setCleanItems] = useState<{ id: string; items: SafeCleanItem[] } | null>(null);
  const [message, setMessage] = useState("");

  const load = useCallback(async () => {
    const [briefs, accs] = await Promise.all([api.getInstanceBriefs(), api.getAccounts()]);
    setBriefs(briefs);
    setAccounts(accs);
    if (accs.length > 0 && !selectedAccount) setSelectedAccount(accs[0].id);
  }, [selectedAccount]);

  useEffect(() => {
    load();
    const interval = setInterval(load, 5000);
    return () => clearInterval(interval);
  }, [load]);

  const handleCreate = async () => {
    if (!name.trim() || !selectedAccount) {
      setError("请输入名称并选择账号");
      return;
    }
    try {
      await api.createInstance(name.trim(), selectedAccount);
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
      await load();
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setLaunching(null);
    }
  };

  const handleStop = async (id: string) => {
    try {
      await api.stopInstance(id);
      await load();
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleRemove = async (id: string, deleteData: boolean) => {
    try {
      await api.removeInstance(id, deleteData);
      await load();
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleAutoDiscover = async () => {
    try {
      const found = await api.autoDiscoverInstances();
      if (found.length > 0) {
        setMessage(`发现 ${found.length} 个新实例`);
      } else {
        setMessage("未发现新实例");
      }
      await load();
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleSaveEdit = async (id: string) => {
    try {
      if (editName.trim()) await api.renameInstance(id, editName.trim());
      await api.updateInstanceNote(id, editNote.trim() || null);
      setEditingId(null);
      await load();
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleShortcut = async (id: string) => {
    try {
      const path = await api.createInstanceShortcut(id);
      setMessage(`快捷方式已创建: ${path}`);
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleClean = async (id: string) => {
    try {
      const items = await api.getSafeCleanItems(id);
      setCleanItems({ id, items });
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleDoClean = async (keys: string[]) => {
    if (!cleanItems) return;
    try {
      const freed = await api.safeCleanInstance(cleanItems.id, keys);
      setMessage(`清理完成，释放 ${formatBytes(freed)}`);
      setCleanItems(null);
      await load();
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const accountName = (id: string) => accounts.find((a) => a.id === id)?.name ?? "未绑定";

  return (
    <div className="p-8 max-w-4xl">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-xl font-semibold text-white">实例管理</h2>
          <p className="text-sm text-gray-500 mt-1">
            每个实例独立 user-data-dir，支持自动发现已有 TRAE 实例
          </p>
        </div>
        <div className="flex gap-2">
          <button className="btn-ghost text-sm" onClick={handleAutoDiscover}>
            自动发现
          </button>
          <button className="btn-primary text-sm" onClick={() => setShowForm(!showForm)}>
            {showForm ? "取消" : "创建实例"}
          </button>
        </div>
      </div>

      {message && (
        <div className="mb-4 px-4 py-2.5 rounded-lg bg-white/5 border border-white/10 text-gray-300 text-sm">
          {message}
        </div>
      )}
      {error && (
        <div className="mb-4 px-4 py-2.5 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
          {error}
        </div>
      )}

      {showForm && (
        <div className="card mb-6 space-y-3">
          <div>
            <label className="text-xs text-gray-400 block mb-1">关联账号</label>
            <select className="input" value={selectedAccount} onChange={(e) => setSelectedAccount(e.target.value)}>
              {accounts.map((a) => (
                <option key={a.id} value={a.id}>{a.name}</option>
              ))}
            </select>
          </div>
          <div>
            <label className="text-xs text-gray-400 block mb-1">实例名称</label>
            <input className="input" value={name} onChange={(e) => setName(e.target.value)} placeholder="如：工作实例" />
          </div>
          <button className="btn-primary" onClick={handleCreate}>确认创建</button>
        </div>
      )}

      {briefs.length === 0 ? (
        <div className="card text-center py-12 text-gray-500">
          <p className="text-sm">还没有实例</p>
          <p className="text-xs text-gray-600 mt-1">点击"自动发现"扫描已有 TRAE，或手动创建</p>
        </div>
      ) : (
        <div className="space-y-3">
          {briefs.map((inst) => {
            const isLaunching = launching === inst.id;
            const isEditing = editingId === inst.id;
            return (
              <div key={inst.id} className="card-hover">
                {isEditing ? (
                  <div className="space-y-3">
                    <input className="input" value={editName} onChange={(e) => setEditName(e.target.value)} placeholder="实例名称" />
                    <input className="input" value={editNote} onChange={(e) => setEditNote(e.target.value)} placeholder="备注" />
                    <div className="flex gap-2">
                      <button className="btn-primary text-xs" onClick={() => handleSaveEdit(inst.id)}>保存</button>
                      <button className="btn-ghost text-xs" onClick={() => setEditingId(null)}>取消</button>
                    </div>
                  </div>
                ) : (
                  <div className="flex items-start justify-between">
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 flex-wrap">
                        <span className="font-medium text-white text-sm">{inst.name}</span>
                        {inst.is_default && <span className="badge-neutral text-xs">默认</span>}
                        <span className="badge-neutral font-mono text-xs">:{inst.debug_port}</span>
                        {inst.is_running ? (
                          <span className="badge-success text-xs">运行中</span>
                        ) : (
                          <span className="badge-neutral text-xs">已停止</span>
                        )}
                        {inst.note && <span className="text-xs text-gray-500">📝 {inst.note}</span>}
                      </div>
                      <div className="mt-1.5 space-y-0.5">
                        <div className="flex items-center gap-3">
                          <span className="text-xs text-gray-500">账号: {accountName(inst.account_id) || "未绑定"}</span>
                          {inst.disk_usage > 0 && (
                            <span className="text-xs text-gray-600 font-mono">{formatBytes(inst.disk_usage)}</span>
                          )}
                        </div>
                        <p className="text-xs text-gray-600 font-mono truncate">{inst.data_dir}</p>
                        {inst.last_launched_at > 0 && (
                          <p className="text-xs text-gray-600">
                            启动: {formatRelativeTime(inst.last_launched_at)}
                            {inst.last_closed_at > 0 && !inst.is_running && ` | 关闭: ${formatRelativeTime(inst.last_closed_at)}`}
                          </p>
                        )}
                      </div>
                    </div>
                    <div className="flex items-center gap-1.5 shrink-0 flex-wrap justify-end">
                      {inst.is_running ? (
                        <button className="btn-danger text-xs" onClick={() => handleStop(inst.id)}>停止</button>
                      ) : (
                        <button className="btn-primary text-xs" onClick={() => handleLaunch(inst.id)} disabled={isLaunching}>
                          {isLaunching ? "启动中..." : "启动"}
                        </button>
                      )}
                      <button className="btn-ghost text-xs" onClick={() => { setEditingId(inst.id); setEditName(inst.name); setEditNote(inst.note || ""); }}>
                        编辑
                      </button>
                      <button className="btn-ghost text-xs" onClick={() => handleShortcut(inst.id)}>快捷方式</button>
                      <button className="btn-ghost text-xs" onClick={() => handleClean(inst.id)}>清理</button>
                      <button className="btn-ghost text-xs" onClick={() => handleRemove(inst.id, false)}>删除</button>
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}

      {cleanItems && (
        <div className="fixed inset-0 bg-black/60 flex items-center justify-center z-50" onClick={() => setCleanItems(null)}>
          <div className="card max-w-md w-full" onClick={(e) => e.stopPropagation()}>
            <h3 className="text-sm font-medium text-white mb-4">安全清理 - 可释放空间</h3>
            <div className="space-y-2 mb-4">
              {cleanItems.items.length === 0 ? (
                <p className="text-xs text-gray-500">没有可清理的项目</p>
              ) : (
                cleanItems.items.map((item) => (
                  <label key={item.key} className="flex items-center justify-between text-sm">
                    <div className="flex items-center gap-2">
                      <input type="checkbox" defaultChecked className="accent-emerald-500" id={`clean-${item.key}`} />
                      <span className="text-gray-300">{item.label}</span>
                      <span className="text-xs text-gray-600">{item.category}</span>
                    </div>
                    <span className="text-xs text-gray-500 font-mono">{formatBytes(item.size)}</span>
                  </label>
                ))
              )}
            </div>
            <div className="flex gap-2">
              <button className="btn-primary text-xs" onClick={() => {
                const keys = cleanItems.items.map((i) => i.key);
                handleDoClean(keys);
              }}>确认清理</button>
              <button className="btn-ghost text-xs" onClick={() => setCleanItems(null)}>取消</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
