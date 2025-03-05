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
   * Debug-Funktion zum Protokollieren von Argumenten
   */
  private static logArgs(
    command: string,
    args?: Record<string, unknown>
  ): void {
    console.log(`Calling ${command} with args:`, JSON.stringify(args, null, 2));
  }

  /**
   * Invoke a Tauri command with debug logging
   */
  static async invoke<T>(
    command: string,
    args?: Record<string, unknown>
  ): Promise<T> {
    // Log the command and args
    this.logArgs(command, args);

    try {
      // Try with args as is
      return await invoke<T>(command, args);
    } catch (err) {
      console.error(`Error invoking ${command}:`, err);

      // Try alternate argument names if the error seems related to arg naming
      const errorMsg = String(err);
      if (errorMsg.includes("missing required key") && args) {
        console.warn("Attempting with alternate argument names...");

        // Try with camelCase keys
        if (command === "set_current_shop_command" && args.shop_id) {
          console.warn("Trying with shopId instead of shop_id");
          const altArgs = { shopId: args.shop_id };
          this.logArgs(`${command} (alternative)`, altArgs);
          try {
            return await invoke<T>(command, altArgs);
          } catch (altErr) {
            console.error("Alternative also failed:", altErr);
          }
        }

        // Try with both versions of the key
        if (command === "set_current_shop_command" && args.shop_id) {
          console.warn("Trying with both shopId and shop_id");
          const dualArgs = {
            shopId: args.shop_id,
            shop_id: args.shop_id,
          };
          this.logArgs(`${command} (dual keys)`, dualArgs);
          try {
            return await invoke<T>(command, dualArgs);
          } catch (dualErr) {
            console.error("Dual keys also failed:", dualErr);
          }
        }

        // Try with flat params when the command seems to expect this
        if (command === "set_current_shop_command" && args.shop_id) {
          const shopId = args.shop_id as string;
          console.warn("Trying with shopId as direct parameter");
          this.logArgs(`${command} (direct param)`, { directParam: shopId });
          try {
            // @ts-ignore - Bypassing type checking for this workaround
            return await invoke<T>(command, shopId);
          } catch (paramErr) {
            console.error("Direct parameter also failed:", paramErr);
          }
        }
      }

      // If nothing worked, rethrow the original error
      throw err;
    }
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
