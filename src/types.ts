export interface Account {
  id: string;
  name: string;
  email: string;
  note: string | null;
  user_id?: string;
  jwt_token?: string;
  refresh_token?: string;
  token_expired_at?: string | null;
  avatar_url?: string;
  plan_type?: string;
  source?: string;
  cookies?: string;
  created_at: number;
  updated_at: number;
}

export interface TraeInstance {
  id: string;
  name: string;
  account_id: string;
  data_dir: string;
  debug_port: number;
  note?: string | null;
  is_default?: boolean;
  machine_id?: string | null;
  last_launched_at?: number;
  last_closed_at?: number;
  created_at: number;
  updated_at: number;
}

export interface InstanceBrief {
  id: string;
  name: string;
  data_dir: string;
  debug_port: number;
  account_id: string;
  note: string | null;
  is_default: boolean;
  is_running: boolean;
  pid: number | null;
  disk_usage: number;
  last_launched_at: number;
  last_closed_at: number;
  created_at: number;
}

export type CheckinStatus =
  | "pending"
  | "in_progress"
  | "success"
  | "already_signed"
  | "not_logged_in"
  | "failed";

export interface CheckinRecord {
  id: string;
  account_id: string;
  instance_id: string;
  status: CheckinStatus;
  detail: string;
  points: number | null;
  checkin_time: number | null;
  created_at: number;
}

export interface CheckinEvent {
  account_id: string;
  account_name: string;
  status: CheckinStatus;
  detail: string;
  points: number | null;
}

export interface BatchSummary {
  total: number;
  success: number;
  already_signed: number;
  failed: number;
  skipped: number;
}

export interface SafeCleanItem {
  key: string;
  label: string;
  category: string;
  path: string;
  size: number;
}

export const statusLabels: Record<CheckinStatus, string> = {
  pending: "待签到",
  in_progress: "签到中",
  success: "已签到",
  already_signed: "今日已签",
  not_logged_in: "未登录",
  failed: "失败",
};

export const statusColors: Record<CheckinStatus, string> = {
  pending: "badge-neutral",
  in_progress: "badge-info",
  success: "badge-success",
  already_signed: "badge-success",
  not_logged_in: "badge-failed",
  failed: "badge-failed",
};

export function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

export function formatRelativeTime(ts: number): string {
  if (!ts || ts === 0) return "-";
  const diff = Date.now() / 1000 - ts;
  if (diff < 60) return "刚刚";
  if (diff < 3600) return `${Math.floor(diff / 60)}分钟前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}小时前`;
  return `${Math.floor(diff / 86400)}天前`;
}
