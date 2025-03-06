// src/lib/services/TauriApiService.ts

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  AppConfig,
  ShopConfig,
  SyncStats,
  VirtueMartOrder,
} from "../types";

export class TauriApiService {
  /**
   * Debug logging for arguments
   */
  private static logArgs(
    command: string,
    args?: Record<string, unknown>
  ): void {
    console.log(`Calling ${command} with args:`, JSON.stringify(args, null, 2));
  }

  /**
   * Invoke a Tauri command with proper parameter casing
   */
  static async invoke<T>(
    command: string,
    params?: Record<string, any>
  ): Promise<T> {
    if (params) {
      // Erstellen Sie eine neue Params-Kopie mit beiden Namenskonventionen
      const enhancedParams: Record<string, any> = {};

      for (const [key, value] of Object.entries(params)) {
        // Original-Parameter beibehalten
        enhancedParams[key] = value;

        // Zusätzlich camelCase in snake_case umwandeln
        if (/[A-Z]/.test(key)) {
          const snakeKey = key.replace(
            /[A-Z]/g,
            (letter) => `_${letter.toLowerCase()}`
          );
          enhancedParams[snakeKey] = value;
        }

        // Zusätzlich snake_case in camelCase umwandeln
        if (key.includes("_")) {
          const camelKey = key.replace(/_([a-z])/g, (_, letter) =>
            letter.toUpperCase()
          );
          enhancedParams[camelKey] = value;
        }
      }

      // Debug-Ausgabe
      console.log(`Enhanced params for ${command}:`, enhancedParams);

      return await invoke<T>(command, enhancedParams);
    }

    return await invoke<T>(command);
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
    return this.invoke<void>("show_notification_command", {
      notification: options,
    });
  }

  /**
   * Abort the current synchronization
   */
  static async abortSync(): Promise<void> {
    return this.invoke<void>("abort_sync_command");
  }

  /**
   * Get system information
   */
  static async getSystemInfo(): Promise<any> {
    return this.invoke("get_system_info");
  }

  /**
   * Get sync statistics for a specific shop or all shops
   */
  static async getSyncStats(shopId?: string): Promise<SyncStats> {
    return this.invoke<SyncStats>(
      "get_sync_stats",
      shopId ? { shop_id: shopId } : {}
    );
  }

  /**
   * Set sync hours for a specific shop
   */
  static async setSyncHours(shopId: string, hours: number): Promise<SyncStats> {
    return this.invoke<SyncStats>("set_sync_hours", {
      shop_id: shopId,
      hours,
    });
  }

  /**
   * Start the synchronization process for a specific shop
   */
  static async startSync(shopId?: string, hours?: number): Promise<void> {
    return this.invoke<void>(
      "start_sync_command",
      shopId ? { shop_id: shopId, hours } : {}
    );
  }

  /**
   * Start synchronization for multiple shops
   */
  static async startMultiSync(shopIds: string[]): Promise<void> {
    return this.invoke<void>("start_multi_sync_command", { shop_ids: shopIds });
  }

  /**
   * Get synchronized orders for a specific shop or all shops
   */
  static async getSyncedOrders(shopId?: string): Promise<VirtueMartOrder[]> {
    return this.invoke<VirtueMartOrder[]>(
      "get_synced_orders",
      shopId ? { shop_id: shopId } : {}
    );
  }

  /**
   * Schedule a sync operation
   */
  static async scheduleSync(
    shopIds: string[],
    cronExpression: string
  ): Promise<void> {
    return this.invoke<void>("schedule_sync", {
      shop_ids: shopIds,
      cron_expression: cronExpression,
    });
  }

  /**
   * Start a scheduled sync operation
   */
  static async startScheduledSync(
    shopIds: string[],
    jobId: string
  ): Promise<void> {
    return this.invoke<void>("start_scheduled_sync", {
      shop_ids: shopIds,
      job_id: jobId,
    });
  }

  /**
   * Cancel scheduled sync jobs for a specific shop or all shops
   */
  static async cancelScheduledSync(shopId?: string): Promise<void> {
    return this.invoke<void>(
      "cancel_scheduled_sync",
      shopId ? { shop_id: shopId } : {}
    );
  }

  /**
   * Load configuration
   */
  static async loadConfig(): Promise<AppConfig> {
    return this.invoke<AppConfig>("load_config_command");
  }

  /**
   * Save configuration
   */
  static async saveConfig(config: AppConfig): Promise<void> {
    return this.invoke<void>("save_config_command", { config });
  }

  /**
   * Add a new shop
   */
  static async addShop(shop: ShopConfig): Promise<AppConfig> {
    return this.invoke<AppConfig>("add_shop_command", { shop });
  }

  /**
   * Update an existing shop
   */
  static async updateShop(shop: ShopConfig): Promise<AppConfig> {
    return this.invoke<AppConfig>("update_shop_command", { shop });
  }

  /**
   * Remove a shop
   */
  static async removeShop(shopId: string): Promise<AppConfig> {
    return this.invoke<AppConfig>("remove_shop_command", { shop_id: shopId });
  }

  /**
   * Convert parameters from camelCase to snake_case for Rust backend compatibility
   */
  private static convertParamsToCasing(
    params: Record<string, any>
  ): Record<string, any> {
    const result: Record<string, any> = {};

    for (const [key, value] of Object.entries(params)) {
      // Convert camelCase to snake_case
      const snakeCaseKey = key.replace(
        /[A-Z]/g,
        (letter) => `_${letter.toLowerCase()}`
      );
      result[snakeCaseKey] = value;

      // Debug the conversion
      if (snakeCaseKey !== key) {
        console.log(`Parameter conversion: ${key} -> ${snakeCaseKey}`);
      }
    }

    return result;
  }

  /**
   * Set current active shop
   */
  static async setCurrentShop(shopId: string): Promise<AppConfig> {
    console.warn("Setting current shop with shopId:", shopId);

    // Try with all possible argument formats to overcome potential issues
    try {
      // Try with snake_case (correct format according to Rust code)
      return await this.invoke<AppConfig>("set_current_shop_command", {
        shop_id: shopId,
      });
    } catch (err1) {
      console.error("Failed with snake_case argument:", err1);

      try {
        // Try with camelCase
        return await invoke<AppConfig>("set_current_shop_command", {
          shopId: shopId,
        });
      } catch (err2) {
        console.error("Failed with camelCase argument:", err2);

        try {
          // Try with both formats
          return await invoke<AppConfig>("set_current_shop_command", {
            shop_id: shopId,
            shopId: shopId,
          });
        } catch (err3) {
          console.error("Failed with both argument formats:", err3);
          throw new Error(
            `Failed to set current shop using all argument formats: ${err3}`
          );
        }
      }
    }
  }
}
