import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Account, TraeInstance, CheckinRecord, CheckinEvent, BatchSummary } from "./types";

export function getAccounts(): Promise<Account[]> {
  return invoke<Account[]>("get_accounts");
}

export function addAccount(
  name: string,
  email: string,
  note: string | null
): Promise<Account> {
  return invoke<Account>("add_account", { name, email, note });
}

export function removeAccount(id: string): Promise<void> {
  return invoke<void>("remove_account", { id });
}

export function updateAccount(
  id: string,
  name: string,
  email: string,
  note: string | null
): Promise<Account> {
  return invoke<Account>("update_account", { id, name, email, note });
}

export function getInstances(): Promise<TraeInstance[]> {
  return invoke<TraeInstance[]>("get_instances");
}

export function createInstance(
  name: string,
  accountId: string
): Promise<TraeInstance> {
  return invoke<TraeInstance>("create_instance", {
    name,
    accountId,
  });
}

export function removeInstance(id: string): Promise<void> {
  return invoke<void>("remove_instance", { id });
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

export function performCheckin(accountId: string): Promise<CheckinRecord> {
  return invoke<CheckinRecord>("perform_checkin", { accountId });
}

export function batchCheckin(): Promise<BatchSummary> {
  return invoke<BatchSummary>("batch_checkin");
}

export function getCheckinRecords(): Promise<CheckinRecord[]> {
  return invoke<CheckinRecord[]>("get_checkin_records");
}

export function findTraePath(): Promise<string> {
  return invoke<string>("find_trae_path");
}

export function setupScheduledTask(hour: number, minute: number): Promise<void> {
  return invoke<void>("setup_scheduled_task", { hour, minute });
}

export function removeScheduledTask(): Promise<void> {
  return invoke<void>("remove_scheduled_task");
}

export function getScheduledTaskStatus(): Promise<boolean> {
  return invoke<boolean>("get_scheduled_task_status");
}

export function onCheckinStatus(callback: (event: CheckinEvent) => void) {
  return listen<CheckinEvent>("checkin-status", (e) => callback(e.payload));
}
