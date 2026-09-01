import { useState, useEffect, useCallback } from "react";
import type { Account, TraeInstance, CheckinRecord, CheckinStatus } from "../types";
import { statusLabels, statusColors } from "../types";
import * as api from "../api";

export default function CheckinPage() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [instances, setInstances] = useState<TraeInstance[]>([]);
  const [records, setRecords] = useState<CheckinRecord[]>([]);
  const [checkingIds, setCheckingIds] = useState<Set<string>>(new Set());
  const [results, setResults] = useState<Record<string, CheckinRecord>>({});
  const [error, setError] = useState("");

  const load = useCallback(async () => {
    const [accs, insts, recs] = await Promise.all([
      api.getAccounts(),
      api.getInstances(),
      api.getCheckinRecords(),
    ]);
    setAccounts(accs);
    setInstances(insts);
    setRecords(recs);

    const resultMap: Record<string, CheckinRecord> = {};
    const today = new Date().toDateString();
    for (const r of recs) {
      if (r.checkin_time && new Date(r.checkin_time * 1000).toDateString() === today) {
        if (!resultMap[r.account_id] || r.created_at > resultMap[r.account_id].created_at) {
          resultMap[r.account_id] = r;
        }
      }
    }
    setResults(resultMap);
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const getInstance = (accountId: string) =>
    instances.find((i) => i.account_id === accountId);

  const handleCheckin = async (accountId: string) => {
    setCheckingIds((prev) => new Set(prev).add(accountId));
    setError("");
    try {
      const record = await api.performCheckin(accountId);
      setResults((prev) => ({ ...prev, [accountId]: record }));
      await load();
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setCheckingIds((prev) => {
        const next = new Set(prev);
        next.delete(accountId);
        return next;
      });
    }
  };

  const todayRecords = records
    .filter((r) => {
      if (!r.checkin_time) return false;
      return new Date(r.checkin_time * 1000).toDateString() === new Date().toDateString();
    })
    .sort((a, b) => b.created_at - a.created_at);

  const formatTime = (ts: number | null) => {
    if (!ts) return "-";
    const d = new Date(ts * 1000);
    return d.toLocaleTimeString("zh-CN", {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  };

  const getStatusBadge = (accountId: string) => {
    const record = results[accountId];
    if (checkingIds.has(accountId)) {
      return <span className={statusColors.in_progress}>{statusLabels.in_progress}</span>;
    }
    if (record) {
      return <span className={statusColors[record.status]}>{statusLabels[record.status]}</span>;
    }
    return <span className="badge-neutral">{statusLabels.pending}</span>;
  };

  return (
    <div className="p-8 max-w-4xl">
      <div className="mb-6">
        <h2 className="text-xl font-semibold text-white">签到中心</h2>
        <p className="text-sm text-gray-500 mt-1">
          选择账号执行签到，自动启动 TRAE 实例并完成 CDP 签到流程
        </p>
      </div>

      {error && (
        <div className="mb-4 px-4 py-2.5 rounded-lg bg-red-500/10 border border-red-500/20 text-red-400 text-sm">
          {error}
        </div>
      )}

      {accounts.length === 0 ? (
        <div className="card text-center py-12 text-gray-500">
          <p className="text-sm">还没有添加账号</p>
          <p className="text-xs text-gray-600 mt-1">
            请先到"账号管理"添加账号和实例
          </p>
        </div>
      ) : (
        <div className="space-y-3 mb-8">
          {accounts.map((acc) => {
            const inst = getInstance(acc.id);
            const isChecking = checkingIds.has(acc.id);
            const record = results[acc.id];
            return (
              <div key={acc.id} className="card-hover">
                <div className="flex items-center justify-between">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2.5">
                      <span className="font-medium text-white text-sm">
                        {acc.name}
                      </span>
                      {getStatusBadge(acc.id)}
                    </div>
                    <div className="flex items-center gap-3 mt-1.5">
                      {inst ? (
                        <>
                          <span className="text-xs text-gray-500 font-mono">
                            :{inst.debug_port}
                          </span>
                          <span className="text-xs text-gray-600">
                            {inst.data_dir.split("\\").pop()}
                          </span>
                        </>
                      ) : (
                        <span className="text-xs text-amber-500/80">
                          未关联实例
                        </span>
                      )}
                    </div>
                    {record && (
                      <div className="flex items-center gap-3 mt-1.5">
                        <span className="text-xs text-gray-500">
                          {record.detail}
                        </span>
                        {record.points && (
                          <span className="text-xs text-emerald-400 font-mono">
                            +{record.points} 积分
                          </span>
                        )}
                        <span className="text-xs text-gray-600 font-mono">
                          {formatTime(record.checkin_time)}
                        </span>
                      </div>
                    )}
                  </div>
                  <button
                    className="btn-primary text-xs"
                    onClick={() => handleCheckin(acc.id)}
                    disabled={isChecking || !inst}
                  >
                    {isChecking ? "签到中..." : "签到"}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {todayRecords.length > 0 && (
        <div>
          <h3 className="text-sm font-medium text-gray-300 mb-3">今日签到记录</h3>
          <div className="card !p-0 overflow-hidden">
            <table className="w-full text-sm">
              <thead>
                <tr className="border-b border-white/5">
                  <th className="text-left text-xs text-gray-500 font-medium px-4 py-2.5">
                    账号
                  </th>
                  <th className="text-left text-xs text-gray-500 font-medium px-4 py-2.5">
                    状态
                  </th>
                  <th className="text-left text-xs text-gray-500 font-medium px-4 py-2.5">
                    详情
                  </th>
                  <th className="text-right text-xs text-gray-500 font-medium px-4 py-2.5">
                    积分
                  </th>
                  <th className="text-right text-xs text-gray-500 font-medium px-4 py-2.5">
                    时间
                  </th>
                </tr>
              </thead>
              <tbody>
                {todayRecords.map((r) => {
                  const acc = accounts.find((a) => a.id === r.account_id);
                  return (
                    <tr
                      key={r.id}
                      className="border-b border-white/5 last:border-0"
                    >
                      <td className="px-4 py-2.5 text-gray-300">
                        {acc?.name ?? "未知"}
                      </td>
                      <td className="px-4 py-2.5">
                        <span className={statusColors[r.status as CheckinStatus]}>
                          {statusLabels[r.status as CheckinStatus]}
                        </span>
                      </td>
                      <td className="px-4 py-2.5 text-gray-500 text-xs">
                        {r.detail}
                      </td>
                      <td className="px-4 py-2.5 text-right text-gray-400 font-mono">
                        {r.points ? `+${r.points}` : "-"}
                      </td>
                      <td className="px-4 py-2.5 text-right text-gray-500 font-mono text-xs">
                        {formatTime(r.checkin_time)}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </div>
  );
}
