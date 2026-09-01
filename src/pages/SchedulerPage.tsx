import { useState, useEffect } from "react";
import * as api from "../api";

export default function SchedulerPage() {
  const [enabled, setEnabled] = useState(false);
  const [hour, setHour] = useState(9);
  const [minute, setMinute] = useState(0);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");

  useEffect(() => {
    api.getScheduledTaskStatus().then(setEnabled).catch(() => {});
  }, []);

  const handleSave = async () => {
    setSaving(true);
    setMessage("");
    try {
      if (enabled) {
        await api.setupScheduledTask(hour, minute);
        setMessage(`定时任务已创建，每天 ${String(hour).padStart(2, "0")}:${String(minute).padStart(2, "0")} 自动签到`);
      } else {
        await api.removeScheduledTask();
        setMessage("定时任务已移除");
      }
    } catch (e: any) {
      setMessage(`操作失败: ${e.toString()}`);
    } finally {
      setSaving(false);
    }
  };

  const handleToggle = async () => {
    const newState = !enabled;
    setEnabled(newState);
    if (!newState) {
      setSaving(true);
      try {
        await api.removeScheduledTask();
        setMessage("定时任务已移除");
      } catch (e: any) {
        setMessage(`移除失败: ${e.toString()}`);
      } finally {
        setSaving(false);
      }
    }
  };

  return (
    <div className="p-8 max-w-2xl">
      <div className="mb-6">
        <h2 className="text-xl font-semibold text-white">定时签到设置</h2>
        <p className="text-sm text-gray-500 mt-1">
          设置 Windows 定时任务，每天自动执行批量签到
        </p>
      </div>

      {message && (
        <div className="mb-4 px-4 py-2.5 rounded-lg bg-white/5 border border-white/10 text-gray-300 text-sm">
          {message}
        </div>
      )}

      <div className="card">
        <div className="flex items-center justify-between mb-6">
          <div>
            <p className="text-sm font-medium text-white">启用定时签到</p>
            <p className="text-xs text-gray-500 mt-0.5">
              关闭后将移除 Windows 计划任务
            </p>
          </div>
          <button
            onClick={handleToggle}
            className={`relative w-11 h-6 rounded-full transition-colors ${
              enabled ? "bg-emerald-500" : "bg-white/10"
            }`}
          >
            <span
              className={`absolute top-0.5 left-0.5 w-5 h-5 rounded-full bg-white transition-transform ${
                enabled ? "translate-x-5" : ""
              }`}
            />
          </button>
        </div>

        {enabled && (
          <div className="space-y-4">
            <div>
              <label className="block text-xs text-gray-500 mb-2">
                签到时间
              </label>
              <div className="flex items-center gap-2">
                <select
                  value={hour}
                  onChange={(e) => setHour(Number(e.target.value))}
                  className="bg-bg-tertiary border border-white/10 rounded-lg px-3 py-2 text-sm text-white font-mono"
                >
                  {Array.from({ length: 24 }, (_, i) => (
                    <option key={i} value={i}>
                      {String(i).padStart(2, "0")}
                    </option>
                  ))}
                </select>
                <span className="text-gray-500">:</span>
                <select
                  value={minute}
                  onChange={(e) => setMinute(Number(e.target.value))}
                  className="bg-bg-tertiary border border-white/10 rounded-lg px-3 py-2 text-sm text-white font-mono"
                >
                  {Array.from({ length: 12 }, (_, i) => i * 5).map((m) => (
                    <option key={m} value={m}>
                      {String(m).padStart(2, "0")}
                    </option>
                  ))}
                </select>
                <span className="text-xs text-gray-500 ml-2">
                  每天 {String(hour).padStart(2, "0")}:{String(minute).padStart(2, "0")} 自动执行
                </span>
              </div>
            </div>

            <div className="pt-2">
              <button
                onClick={handleSave}
                disabled={saving}
                className="btn-primary text-sm px-5 py-2"
              >
                {saving ? "保存中..." : "保存设置"}
              </button>
            </div>
          </div>
        )}
      </div>

      <div className="mt-4 card">
        <p className="text-xs text-gray-500 leading-relaxed">
          定时任务通过 Windows 计划任务 (schtasks) 实现，将在指定时间自动启动应用并执行批量签到。
          确保应用安装在固定路径下，且 TRAE 已正确安装。
          签到过程中会自动跳过今日已签到的账号。
        </p>
      </div>
    </div>
  );
}
