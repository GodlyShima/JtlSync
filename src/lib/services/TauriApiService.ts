import type { AppConfig, SyncStats, SystemInfo } from "$lib/types";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/**
 * Service for interacting with the Tauri backend
 */
export class TauriApiService {
  // Event listeners
  private static listeners: Map<string, Function[]> = new Map();

  /**
   * Initialize the service and set up event listeners
   */
  static async initialize(): Promise<void> {
    // Listen for events from the backend
    await listen("jtl-api-status", (event) => {
      this.notifyListeners("jtl-api-status", event.payload);
    });

    await listen("sync-progress", (event) => {
      this.notifyListeners("sync-progress", event.payload);
    });

    await listen("log", (event) => {
      this.notifyListeners("log", event.payload);
    });
  }

  /**
   * Add an event listener
   * @param event The event to listen for
   * @param callback The callback function
   */
  static addEventListener(event: string, callback: Function): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)?.push(callback);
  }

  /**
   * Remove an event listener
   * @param event The event to stop listening for
   * @param callback The callback function to remove
   */
  static removeEventListener(event: string, callback: Function): void {
    if (this.listeners.has(event)) {
      const callbacks = this.listeners.get(event) || [];
      const index = callbacks.indexOf(callback);
      if (index !== -1) {
        callbacks.splice(index, 1);
      }
    }
  }

  /**
   * Notify all listeners of an event
   * @param event The event that occurred
   * @param data The event data
   */
  private static notifyListeners(event: string, data: any): void {
    if (this.listeners.has(event)) {
      for (const callback of this.listeners.get(event) || []) {
        callback(data);
      }
    }
  }

  /**
   * Start the JTL API service
   * @param config The application configuration
   */
  static async startJtlApi(config: AppConfig): Promise<string> {
    return invoke("start_jtl_api", { config });
  }

  /**
   * Stop the JTL API service
   */
  static async stopJtlApi(): Promise<string> {
    return invoke("stop_jtl_api");
  }

  /**
   * Check if the JTL API service is running
   */
  static async isJtlApiRunning(): Promise<boolean> {
    return invoke("is_jtl_api_running");
  }

  /**
   * Get system information
   */
  static async getSystemInfo(): Promise<SystemInfo> {
    return invoke("get_system_info");
  }

  /**
   * Save the application configuration
   * @param config The configuration to save
   */
  static async saveConfig(config: AppConfig): Promise<void> {
    return invoke("save_config", { config });
  }

  /**
   * Load the application configuration
   */
  static async loadConfig(): Promise<AppConfig> {
    return invoke("load_config");
  }

  /**
   * Start a manual synchronization
   */
  static async startSync(): Promise<SyncStats> {
    // This will be implemented in the Rust backend
    return invoke("start_sync");
  }

  /**
   * Schedule a synchronization job
   * @param cronExpression The cron expression for scheduling
   */
  static async scheduleSync(cronExpression: string): Promise<void> {
    return invoke("schedule_sync", { cronExpression });
  }

  /**
   * Cancel all scheduled synchronization jobs
   */
  static async cancelScheduledSync(): Promise<void> {
    return invoke("cancel_scheduled_sync");
  }

  /**
   * Get the current synchronization statistics
   */
  static async getSyncStats(): Promise<SyncStats> {
    return invoke("get_sync_stats");
  }
}

// Initialize the service when the module is imported
TauriApiService.initialize().catch(console.error);
