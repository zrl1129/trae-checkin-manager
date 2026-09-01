export interface Account {
  id: string;
  name: string;
  email: string;
  note: string | null;
  created_at: number;
  updated_at: number;
}

export interface TraeInstance {
  id: string;
  name: string;
  account_id: string;
  data_dir: string;
  debug_port: number;
  created_at: number;
  updated_at: number;
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

export const statusLabels: Record<CheckinStatus, string> = {
  pending: "待签到",
  in_progress: "签到中",
  success: "已签到",
  already_signed: "今日已签",
  not_logged_in: "未登录",
  failed: "失败",
};

export const statusColors: Record<CheckinStatus, string> = {
  pending: "badge-pending",
  in_progress: "badge-info",
  success: "badge-success",
  already_signed: "badge-success",
  not_logged_in: "badge-failed",
  failed: "badge-failed",
};
