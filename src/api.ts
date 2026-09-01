import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  Account, TraeInstance, InstanceBrief, CheckinRecord, CheckinEvent, BatchSummary, SafeCleanItem,
} from "./types";

// === Accounts ===
export function getAccounts(): Promise<Account[]> {
  return invoke<Account[]>("get_accounts");
}
export function addAccount(name: string, email: string, note: string | null): Promise<Account> {
  return invoke<Account>("add_account", { name, email, note });
}
export function removeAccount(id: string): Promise<void> {
  return invoke<void>("remove_account", { id });
}
export function updateAccount(id: string, name: string, email: string, note: string | null): Promise<Account> {
  return invoke<Account>("update_account", { id, name, email, note });
}
export function updateAccountNote(id: string, note: string | null): Promise<void> {
  return invoke<void>("update_account_note", { id, note });
}
export function readLocalAccount(): Promise<Account | null> {
  return invoke<Account | null>("read_local_account");
}
export function exportAccounts(): Promise<string> {
  return invoke<string>("export_accounts");
}
export function importAccounts(jsonStr: string, overwrite: boolean): Promise<number> {
  return invoke<number>("import_accounts", { jsonStr, overwrite });
}

// === Instances ===
export function getInstances(): Promise<TraeInstance[]> {
  return invoke<TraeInstance[]>("get_instances");
}
export function getInstanceBriefs(): Promise<InstanceBrief[]> {
  return invoke<InstanceBrief[]>("get_instance_briefs");
}
export function createInstance(name: string, accountId: string): Promise<TraeInstance> {
  return invoke<TraeInstance>("create_instance", { name, accountId });
}
export function removeInstance(id: string, deleteData?: boolean): Promise<void> {
  return invoke<void>("remove_instance", { id, deleteData: deleteData ?? false });
}
export function renameInstance(id: string, newName: string): Promise<void> {
  return invoke<void>("rename_instance", { id, newName });
}
export function updateInstanceNote(id: string, note: string | null): Promise<void> {
  return invoke<void>("update_instance_note", { id, note });
}
export function bindAccountToInstance(id: string, accountId: string | null): Promise<void> {
  return invoke<void>("bind_account_to_instance", { id, accountId });
}
export function launchInstance(id: string): Promise<number> {
  return invoke<number>("launch_instance", { id });
}
export function stopInstance(id: string): Promise<void> {
  return invoke<void>("stop_instance", { id });
}
export function checkInstanceRunning(id: string): Promise<boolean> {
  return invoke<boolean>("check_instance_running", { id });
}
export function autoDiscoverInstances(): Promise<TraeInstance[]> {
  return invoke<TraeInstance[]>("auto_discover_instances");
}
export function createInstanceShortcut(id: string): Promise<string> {
  return invoke<string>("create_instance_shortcut", { id });
}
export function getSafeCleanItems(id: string): Promise<SafeCleanItem[]> {
  return invoke<SafeCleanItem[]>("get_safe_clean_items", { id });
}
export function safeCleanInstance(id: string, keys: string[]): Promise<number> {
  return invoke<number>("safe_clean_instance", { id, keys });
}

// === Checkin ===
export function performCheckin(accountId: string): Promise<CheckinRecord> {
  return invoke<CheckinRecord>("perform_checkin", { accountId });
}
export function batchCheckin(): Promise<BatchSummary> {
  return invoke<BatchSummary>("batch_checkin");
}
export function getCheckinRecords(): Promise<CheckinRecord[]> {
  return invoke<CheckinRecord[]>("get_checkin_records");
}

// === Path ===
export function findTraePath(): Promise<string> {
  return invoke<string>("find_trae_path");
}

// === Scheduler ===
export function setupScheduledTask(hour: number, minute: number): Promise<void> {
  return invoke<void>("setup_scheduled_task", { hour, minute });
}
export function removeScheduledTask(): Promise<void> {
  return invoke<void>("remove_scheduled_task");
}
export function getScheduledTaskStatus(): Promise<boolean> {
  return invoke<boolean>("get_scheduled_task_status");
}

// === Events ===
export function onCheckinStatus(callback: (event: CheckinEvent) => void) {
  return listen<CheckinEvent>("checkin-status", (e) => callback(e.payload));
}
