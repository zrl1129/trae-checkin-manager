import { useState, useEffect } from "react";
import type { Account } from "../types";
import * as api from "../api";

export default function AccountsPage() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [note, setNote] = useState("");
  const [error, setError] = useState("");
  const [message, setMessage] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editNote, setEditNote] = useState("");

  const load = () => api.getAccounts().then(setAccounts);

  useEffect(() => {
    load();
  }, []);

  const handleAdd = async () => {
    if (!name.trim()) {
      setError("请输入账号名称");
      return;
    }
    try {
      await api.addAccount(name.trim(), email.trim(), note.trim() || null);
      setName(""); setEmail(""); setNote(""); setError("");
      setShowForm(false);
      await load();
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleRemove = async (id: string) => {
    try {
      await api.removeAccount(id);
      await load();
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleReadLocal = async () => {
    try {
      const acc = await api.readLocalAccount();
      if (acc) {
        setMessage(`已读取本地账号: ${acc.name}`);
        await load();
      } else {
        setMessage("未检测到已登录的 TRAE 账号");
      }
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleExport = async () => {
    try {
      const json = await api.exportAccounts();
      const blob = new Blob([json], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `trae-accounts-${new Date().toISOString().slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(url);
      setMessage("账号已导出");
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleImport = async (overwrite: boolean) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".json";
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;
      try {
        const text = await file.text();
        const count = await api.importAccounts(text, overwrite);
        setMessage(`导入完成，共 ${count} 个账号`);
        await load();
      } catch (e: any) {
        setError(e.toString());
      }
    };
    input.click();
  };

  const handleSaveNote = async (id: string) => {
    try {
      await api.updateAccountNote(id, editNote.trim() || null);
      setEditingId(null);
      await load();
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const getSourceLabel = (source?: string) => {
    switch (source) {
      case "local": return "本地";
      case "browser": return "浏览器";
      case "manual": return "手动";
      default: return source || "手动";
    }
  };

  return (
    <div className="p-8 max-w-4xl">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-xl font-semibold text-white">账号管理</h2>
          <p className="text-sm text-gray-500 mt-1">管理 TRAE 账号，支持自动读取、导入导出</p>
        </div>
        <div className="flex gap-2">
          <button className="btn-ghost text-sm" onClick={handleReadLocal}>读取本地</button>
          <button className="btn-ghost text-sm" onClick={() => handleImport(false)}>导入</button>
          <button className="btn-ghost text-sm" onClick={handleExport}>导出</button>
          <button className="btn-primary text-sm" onClick={() => setShowForm(!showForm)}>
            {showForm ? "取消" : "添加"}
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
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="text-xs text-gray-400 block mb-1">账号名称</label>
              <input className="input" value={name} onChange={(e) => setName(e.target.value)} placeholder="如：工作账号" />
            </div>
            <div>
              <label className="text-xs text-gray-400 block mb-1">邮箱（选填）</label>
              <input className="input" value={email} onChange={(e) => setEmail(e.target.value)} placeholder="user@example.com" />
            </div>
          </div>
          <div>
            <label className="text-xs text-gray-400 block mb-1">备注（选填）</label>
            <input className="input" value={note} onChange={(e) => setNote(e.target.value)} placeholder="可选备注" />
          </div>
          <button className="btn-primary" onClick={handleAdd}>确认添加</button>
        </div>
      )}

      {accounts.length === 0 ? (
        <div className="card text-center py-12 text-gray-500">
          <p className="text-sm">还没有账号</p>
          <p className="text-xs text-gray-600 mt-1">点击"读取本地"自动检测，或手动添加</p>
        </div>
      ) : (
        <div className="space-y-3">
          {accounts.map((acc) => (
            <div key={acc.id} className="card-hover">
              <div className="flex items-start justify-between">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
                    {acc.avatar_url && (
                      <img src={acc.avatar_url} alt="" className="w-5 h-5 rounded-full" />
                    )}
                    <span className="font-medium text-white text-sm">{acc.name}</span>
                    {acc.plan_type && acc.plan_type !== "Free" && (
                      <span className="badge-info text-xs">{acc.plan_type}</span>
                    )}
                    {acc.source && (
                      <span className="badge-neutral text-xs">{getSourceLabel(acc.source)}</span>
                    )}
                  </div>
                  <div className="mt-1 flex items-center gap-3">
                    {acc.email && <span className="text-xs text-gray-500 font-mono">{acc.email}</span>}
                    {acc.user_id && <span className="text-xs text-gray-600 font-mono">ID:{acc.user_id.slice(0, 8)}</span>}
                  </div>
                  {editingId === acc.id ? (
                    <div className="mt-2 flex gap-2">
                      <input className="input text-xs flex-1" value={editNote} onChange={(e) => setEditNote(e.target.value)} placeholder="备注" />
                      <button className="btn-primary text-xs" onClick={() => handleSaveNote(acc.id)}>保存</button>
                      <button className="btn-ghost text-xs" onClick={() => setEditingId(null)}>取消</button>
                    </div>
                  ) : (
                    acc.note && <p className="text-xs text-gray-500 mt-1 truncate">📝 {acc.note}</p>
                  )}
                </div>
                <div className="flex items-center gap-1.5 shrink-0">
                  <button className="btn-ghost text-xs" onClick={() => { setEditingId(acc.id); setEditNote(acc.note || ""); }}>
                    备注
                  </button>
                  <button className="btn-danger text-xs" onClick={() => handleRemove(acc.id)}>删除</button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
