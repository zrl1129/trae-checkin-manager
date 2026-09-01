import { invoke } from "@tauri-apps/api/core";
import type { Account, TraeInstance, CheckinRecord } from "./types";

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

export function getCheckinRecords(): Promise<CheckinRecord[]> {
  return invoke<CheckinRecord[]>("get_checkin_records");
}

export function findTraePath(): Promise<string> {
  return invoke<string>("find_trae_path");
}
