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
    return invoke<void>("show_notification_command", { notification: options });
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
   * Get sync statistics for a specific shop or all shops
   */
  static async getSyncStats(shopId?: string): Promise<SyncStats> {
    return invoke<SyncStats>(
      "get_sync_stats",
      shopId ? { shop_id: shopId } : {}
    );
  }

  /**
   * Start the synchronization process for a specific shop
   */
  static async startSync(shopId?: string): Promise<void> {
    return invoke<void>(
      "start_sync_command",
      shopId ? { shop_id: shopId } : {}
    );
  }

  /**
   * Get synchronized orders for a specific shop or all shops
   */
  static async getSyncedOrders(shopId?: string): Promise<VirtueMartOrder[]> {
    return invoke<VirtueMartOrder[]>(
      "get_synced_orders",
      shopId ? { shop_id: shopId } : {}
    );
  }

  /**
   * Schedule a sync operation
   */
  static async scheduleSync(
    shopId: string,
    cronExpression: string
  ): Promise<void> {
    return invoke<void>("schedule_sync", {
      shop_id: shopId,
      cron_expression: cronExpression,
    });
  }

  /**
   * Cancel scheduled sync jobs for a specific shop or all shops
   */
  static async cancelScheduledSync(shopId?: string): Promise<void> {
    return invoke<void>(
      "cancel_scheduled_sync",
      shopId ? { shop_id: shopId } : {}
    );
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

  /**
   * Add a new shop
   */
  static async addShop(shop: ShopConfig): Promise<AppConfig> {
    return invoke<AppConfig>("add_shop_command", { shop });
  }

  /**
   * Update an existing shop
   */
  static async updateShop(shop: ShopConfig): Promise<AppConfig> {
    return invoke<AppConfig>("update_shop_command", { shop });
  }

  /**
   * Remove a shop
   */
  static async removeShop(shopId: string): Promise<AppConfig> {
    return invoke<AppConfig>("remove_shop_command", { shop_id: shopId });
  }

  /**
   * Set current active shop
   */
  static async setCurrentShop(shopId: string): Promise<AppConfig> {
    return invoke<AppConfig>("set_current_shop_command", { shop_id: shopId });
  }
}
