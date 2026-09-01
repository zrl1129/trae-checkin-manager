import { useState, useEffect, useCallback } from "react";
import type { Account, TraeInstance, CheckinRecord, CheckinEvent, BatchSummary } from "../types";
import { statusLabels, statusColors } from "../types";
import type { CheckinStatus } from "../types";
import * as api from "../api";

export default function CheckinPage() {
  const [accounts, setAccounts] = useState<Account[]>([]);
  const [instances, setInstances] = useState<TraeInstance[]>([]);
  const [records, setRecords] = useState<CheckinRecord[]>([]);
  const [liveStatus, setLiveStatus] = useState<Record<string, CheckinEvent>>({});
  const [checkingIds, setCheckingIds] = useState<Set<string>>(new Set());
  const [batchRunning, setBatchRunning] = useState(false);
  const [batchSummary, setBatchSummary] = useState<BatchSummary | null>(null);
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
  }, []);

  useEffect(() => {
    load();
    const unlisten = api.onCheckinStatus((event) => {
      setLiveStatus((prev) => ({ ...prev, [event.account_id]: event }));
      if (event.status === "in_progress") {
        setCheckingIds((prev) => new Set(prev).add(event.account_id));
      } else {
        setCheckingIds((prev) => {
          const next = new Set(prev);
          next.delete(event.account_id);
          return next;
        });
        load();
      }
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [load]);

  const getInstance = (accountId: string) =>
    instances.find((i) => i.account_id === accountId);

  const getLatestRecord = useCallback(
    (accountId: string): CheckinRecord | undefined => {
      const today = new Date().toDateString();
      return records
        .filter(
          (r) =>
            r.account_id === accountId &&
            r.checkin_time &&
            new Date(r.checkin_time * 1000).toDateString() === today
        )
        .sort((a, b) => b.created_at - a.created_at)[0];
    },
    [records]
  );

  const handleCheckin = async (accountId: string) => {
    setCheckingIds((prev) => new Set(prev).add(accountId));
    setError("");
    try {
      await api.performCheckin(accountId);
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

  const handleBatchCheckin = async () => {
    setBatchRunning(true);
    setBatchSummary(null);
    setError("");
    try {
      const summary = await api.batchCheckin();
      setBatchSummary(summary);
      await load();
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setBatchRunning(false);
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
    const live = liveStatus[accountId];
    if (checkingIds.has(accountId) || (live && live.status === "in_progress")) {
      return <span className={statusColors.in_progress}>{statusLabels.in_progress}</span>;
    }
    if (live) {
      return <span className={statusColors[live.status as CheckinStatus]}>{statusLabels[live.status as CheckinStatus]}</span>;
    }
    const record = getLatestRecord(accountId);
    if (record) {
      return <span className={statusColors[record.status]}>{statusLabels[record.status]}</span>;
    }
    return <span className="badge-neutral">{statusLabels.pending}</span>;
  };

  const getDetail = (accountId: string) => {
    const live = liveStatus[accountId];
    if (live) return live;
    const record = getLatestRecord(accountId);
    return record
      ? {
          account_id: record.account_id,
          account_name: "",
          status: record.status,
          detail: record.detail,
          points: record.points,
        }
      : null;
  };

  return (
    <div className="p-8 max-w-4xl">
      <div className="mb-6 flex items-center justify-between">
        <div>
          <h2 className="text-xl font-semibold text-white">签到中心</h2>
          <p className="text-sm text-gray-500 mt-1">
            一键批量签到所有未签到账号，实时显示签到状态
          </p>
        </div>
        <button
          className="btn-primary text-sm px-5 py-2.5"
          onClick={handleBatchCheckin}
          disabled={batchRunning || accounts.length === 0}
        >
          {batchRunning ? "批量签到中..." : "一键签到全部"}
        </button>
      </div>

      {batchSummary && (
        <div className="mb-4 px-4 py-3 rounded-lg bg-white/5 border border-white/10">
          <div className="flex items-center gap-6 text-sm">
            <span className="text-gray-400">
              共 <span className="text-white font-mono">{batchSummary.total}</span> 个
            </span>
            <span className="text-emerald-400">
              成功 <span className="font-mono">{batchSummary.success}</span>
            </span>
            <span className="text-blue-400">
              已签 <span className="font-mono">{batchSummary.already_signed}</span>
            </span>
            <span className="text-red-400">
              失败 <span className="font-mono">{batchSummary.failed}</span>
            </span>
            <span className="text-gray-500">
              跳过 <span className="font-mono">{batchSummary.skipped}</span>
            </span>
          </div>
        </div>
      )}

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
            const isChecking = checkingIds.has(acc.id) || (liveStatus[acc.id]?.status === "in_progress");
            const detail = getDetail(acc.id);
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
                    {detail && (
                      <div className="flex items-center gap-3 mt-1.5">
                        <span className="text-xs text-gray-500">
                          {detail.detail}
                        </span>
                        {detail.points && (
                          <span className="text-xs text-emerald-400 font-mono">
                            +{detail.points} 积分
                          </span>
                        )}
                        {getLatestRecord(acc.id) && (
                          <span className="text-xs text-gray-600 font-mono">
                            {formatTime(getLatestRecord(acc.id)!.checkin_time)}
                          </span>
                        )}
                      </div>
                    )}
                  </div>
                  <button
                    className="btn-primary text-xs"
                    onClick={() => handleCheckin(acc.id)}
                    disabled={isChecking || !inst || batchRunning}
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
