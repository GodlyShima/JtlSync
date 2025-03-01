import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AppConfig, SyncStats } from "../types";

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
