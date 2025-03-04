// Update to src/lib/services/TauriApiService.ts

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { AppConfig, SyncStats, VirtueMartOrder } from "../types";

export class TauriApiService {
  /**
   * Invoke a Tauri command
   */
  static async invoke<T>(
    command: string,
    args?: Record<string, unknown>
  ): Promise<T> {
    return invoke<T>(command, args);
  }

  /**
   * Listen for Tauri events
   */
  static async listen(
    event: string,
    callback: (event: any) => void
  ): Promise<UnlistenFn> {
    return listen(event, callback);
  }

  /**
   * Stop listening for a Tauri event
   */
  static async unlisten(
    event: string,
    callback: (event: any) => void
  ): Promise<void> {
    const unlistenFn = await listen(event, callback);
    unlistenFn();
  }

  /**
   * Show notification
   * This will emit an event that our Rust backend needs to handle
   */
  static async showNotification(options: {
    title: string;
    body: string;
  }): Promise<void> {
    // We'll emit an event that our Rust backend will handle
    const appWindow = await getCurrentWindow();
    return appWindow.emit("show-notification", options);
  }

  /**
   * Abort the current synchronization
   */
  static async abortSync(): Promise<void> {
    return invoke<void>("abort_sync_command");
  }

  /**
   * Get system information
   */
  static async getSystemInfo(): Promise<any> {
    return invoke("get_system_info");
  }

  /**
   * Get current sync statistics
   */
  static async getSyncStats(): Promise<SyncStats> {
    return invoke<SyncStats>("get_sync_stats");
  }

  /**
   * Start the synchronization process
   */
  static async startSync(config: AppConfig): Promise<SyncStats> {
    return invoke<SyncStats>("start_sync_command", { config });
  }

  /**
   * Get synchronized orders
   */
  static async getSyncedOrders(): Promise<VirtueMartOrder[]> {
    return invoke<VirtueMartOrder[]>("get_synced_orders");
  }

  /**
   * Schedule a sync operation
   */
  static async scheduleSync(cronExpression: string): Promise<void> {
    return invoke<void>("schedule_sync", { cronExpression });
  }

  /**
   * Cancel scheduled sync jobs
   */
  static async cancelScheduledSync(): Promise<void> {
    return invoke<void>("cancel_scheduled_sync");
  }

  /**
   * Load configuration
   */
  static async loadConfig(): Promise<AppConfig> {
    return invoke<AppConfig>("load_config_command");
  }

  /**
   * Save configuration
   */
  static async saveConfig(config: AppConfig): Promise<void> {
    return invoke<void>("save_config_command", { config });
  }
}
