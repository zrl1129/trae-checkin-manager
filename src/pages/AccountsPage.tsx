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
      setName("");
      setEmail("");
      setNote("");
      setError("");
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

  return (
    <div className="p-8 max-w-4xl">
      <div className="flex items-center justify-between mb-6">
        <div>
          <h2 className="text-xl font-semibold text-white">账号管理</h2>
          <p className="text-sm text-gray-500 mt-1">
            管理你的 TRAE 账号，每个账号可绑定独立实例
          </p>
        </div>
        <button
          className="btn-primary"
          onClick={() => setShowForm(!showForm)}
        >
          {showForm ? "取消" : "添加账号"}
        </button>
      </div>

      {error && (
        <div className="mb-4 px-4 py-2.5 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
          {error}
        </div>
      )}

      {showForm && (
        <div className="card mb-6 space-y-3">
          <div className="grid grid-cols-2 gap-3">
            <div>
              <label className="text-xs text-gray-400 block mb-1">
                账号名称
              </label>
              <input
                className="input"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="如：工作账号"
              />
            </div>
            <div>
              <label className="text-xs text-gray-400 block mb-1">
                邮箱（选填）
              </label>
              <input
                className="input"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                placeholder="user@example.com"
              />
            </div>
          </div>
          <div>
            <label className="text-xs text-gray-400 block mb-1">
              备注（选填）
            </label>
            <input
              className="input"
              value={note}
              onChange={(e) => setNote(e.target.value)}
              placeholder="可选备注信息"
            />
          </div>
          <button className="btn-primary" onClick={handleAdd}>
            确认添加
          </button>
        </div>
      )}

      {accounts.length === 0 ? (
        <div className="card text-center py-12 text-gray-500">
          <p className="text-sm">还没有添加任何账号</p>
          <p className="text-xs text-gray-600 mt-1">
            点击右上角"添加账号"开始
          </p>
        </div>
      ) : (
        <div className="space-y-3">
          {accounts.map((acc) => (
            <div key={acc.id} className="card-hover flex items-center justify-between">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="font-medium text-white text-sm">
                    {acc.name}
                  </span>
                  {acc.email && (
                    <span className="text-xs text-gray-500 font-mono">
                      {acc.email}
                    </span>
                  )}
                </div>
                {acc.note && (
                  <p className="text-xs text-gray-500 mt-1 truncate">{acc.note}</p>
                )}
              </div>
              <button
                className="btn-danger text-xs"
                onClick={() => handleRemove(acc.id)}
              >
                删除
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
