<script lang="ts">
  import { TauriApiService } from "$lib/services/TauriApiService";
  import type { AppConfig } from "$lib/types";
  import { faDatabase, faSync } from "@fortawesome/free-solid-svg-icons";
  import { onDestroy, onMount } from "svelte";
  import Fa from "svelte-fa";
  import PanelHeader from "./PanelHeader.svelte";
  import StatItem from "./StatItem.svelte";
  
  // Default config
  const defaultConfig: AppConfig = {
    joomla: {
      host: "w01539f0.kasserver.com",
      user: "d0243b57",
      password: "mallorca",
      database: "d0243b57"
    },
    jtl: {
      host: "localhost",
      user: "root",
      password: "",
      database: "jtl"
    },
    tables: {
      orders: "y13ci_virtuemart_orders",
      orderItems: "y13ci_virtuemart_order_items",
      customers: "y13ci_virtuemart_order_userinfos"
    },
    logFile: "",
    jtlApiPath: ""  // API path is no longer needed
  };
  
  // Initialize sync stats
  let syncStats = {
    total_orders: 0,
    synced_orders: 0,
    skipped_orders: 0,
    error_orders: 0,
    last_sync_time: null as Date | null,
    next_scheduled_run: null as Date | null
  };
  
  let isLoading = false;
  let error: string | null = null;
  let config: AppConfig = defaultConfig;
  let configLoaded = false;
  
  // Format date/time
  function formatDateTime(date: Date | null): string {
    if (!date) return "Nie";
    return new Intl.DateTimeFormat('de-DE', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    }).format(date);
  }
  
  // Load or create initial config
  async function loadOrCreateConfig() {
    try {
      // Try to load existing config
      const loadedConfig = await TauriApiService.invoke<AppConfig>('load_config_command');
      config = loadedConfig;
      configLoaded = true;
      console.log("Config loaded successfully");
    } catch (err) {
      console.warn("No existing config found, saving default config");
      
      // If no config exists, save the default one
      try {
        await TauriApiService.invoke('save_config_command', { config: defaultConfig });
        config = defaultConfig;
        configLoaded = true;
        console.log("Default config saved successfully");
      } catch (saveErr) {
        console.error("Failed to save default config:", saveErr);
        error = `Fehler beim Speichern der Standardkonfiguration: ${saveErr}`;
      }
    }
  }
  
  // Load initial stats
  async function loadStats() {
    try {
      isLoading = false;
      error = null;
      
      // Load or create config if not already done
      if (!configLoaded) {
        await loadOrCreateConfig();
      }
      
      // Get sync stats
      try {
        const stats = await TauriApiService.invoke<{
          total_orders: number,
          synced_orders: number,
          skipped_orders: number,
          error_orders: number,
          last_sync_time: string | null,
          next_scheduled_run: string | null
        }>('get_sync_stats');
        
        syncStats = {
          total_orders: stats.total_orders,
          synced_orders: stats.synced_orders,
          skipped_orders: stats.skipped_orders,
          error_orders: stats.error_orders,
          // Convert timestamps from strings to Date objects if needed
          last_sync_time: stats.last_sync_time ? new Date(stats.last_sync_time) : null,
          next_scheduled_run: stats.next_scheduled_run ? new Date(stats.next_scheduled_run) : null
        };
      } catch (statsErr) {
        console.warn("Failed to get sync stats, likely because sync hasn't run yet:", statsErr);
      }
    } catch (err) {
      console.error("Failed to load sync stats:", err);
      error = `Fehler beim Laden der Synchronisationsstatistiken: ${err}`;
    } finally {
      isLoading = false;
    }
  }
  
  // Start manual sync
  async function startSync() {
    if (!configLoaded) {
      error = "Keine Konfiguration gefunden. Bitte konfigurieren Sie die Anwendung zuerst.";
      return;
    }
    
    try {
      isLoading = true;
      error = null;
      
      console.log("Starting sync with config:", config);
      await TauriApiService.invoke('start_sync_command', { config });
      
      
    } catch (err) {
      console.error("Failed to start sync:", err);
      error = `Fehler beim Starten der Synchronisation: ${err}`;
      isLoading = false;  // Reset loading state on error
    }
  }
  
  // Handle sync progress updates
  function handleSyncProgress(event: { payload: any }) {
    const stats = event.payload;
    console.log("Sync progress update:", stats);
    
    syncStats = {
      total_orders: stats.total_orders ?? syncStats.total_orders,
      synced_orders: stats.synced_orders ?? syncStats.synced_orders,
      skipped_orders: stats.skipped_orders ?? syncStats.skipped_orders,
      error_orders: stats.error_orders ?? syncStats.error_orders,
      // Convert timestamps from strings to Date objects if needed
      last_sync_time: stats.last_sync_time ? new Date(stats.last_sync_time) : syncStats.last_sync_time,
      next_scheduled_run: stats.next_scheduled_run ? new Date(stats.next_scheduled_run) : syncStats.next_scheduled_run
    };
  }
  
  // Handle sync completion
  function handleSyncComplete(event: { payload: any }) {
    const stats = event.payload;
    console.log("Sync completed:", stats);
    
    syncStats = {
      total_orders: stats.total_orders ?? syncStats.total_orders,
      synced_orders: stats.synced_orders ?? syncStats.synced_orders,
      skipped_orders: stats.skipped_orders ?? syncStats.skipped_orders,
      error_orders: stats.error_orders ?? syncStats.error_orders,
      // Convert timestamps from strings to Date objects if needed
      last_sync_time: stats.last_sync_time ? new Date(stats.last_sync_time) : syncStats.last_sync_time,
      next_scheduled_run: stats.next_scheduled_run ? new Date(stats.next_scheduled_run) : syncStats.next_scheduled_run
    };
    
    isLoading = false;  // Reset loading state only after a short delay
    console.log("Sync loading state reset to false");
  }
  
  // Handle sync error
  function handleSyncError(event: { payload: string }) {
    const err = event.payload;
    console.error("Sync error:", err);
    error = `Fehler bei der Synchronisation: ${err}`;
    
    // Use a small delay to ensure error is displayed
    setTimeout(() => {
      isLoading = false;  // Reset loading state only after a short delay
      console.log("Sync loading state reset to false after error");
    }, 500);
  }
  
  // Set up event listeners
  onMount(async () => {
    console.log("SyncPanel mounted");
    
    // Listen for sync events
    TauriApiService.listen("sync-stats-update", handleSyncProgress);
    TauriApiService.listen("sync-process-complete", handleSyncComplete);
    TauriApiService.listen("sync-error", handleSyncError);
    
    // Load initial data
    await loadStats();
  });
  
  // Clean up
  onDestroy(() => {
    TauriApiService.unlisten("sync-stats-update", handleSyncProgress);
    TauriApiService.unlisten("sync-process-complete", handleSyncComplete);
    TauriApiService.unlisten("sync-error", handleSyncError);
  });
</script>

<div class="stat-panel">
  <PanelHeader icon={faDatabase} title="Synchronisierungsdetails" />
  
  {#if error}
    <div class="error-message">{error}</div>
  {/if}
  
  <div class="system-grid">
    <StatItem color="var(--green)" label="Abgeschlossen" value={syncStats.synced_orders.toString()} />
    <StatItem color="var(--yellow)" label="Übersprungen" value={syncStats.skipped_orders.toString()} />
    <StatItem color="var(--red)" label="Fehlgeschlagen" value={syncStats.error_orders.toString()} />
    <StatItem label="Letzte Synchronisation" value={formatDateTime(syncStats.last_sync_time)} />
    <StatItem label="Nächste Synchronisation" value={formatDateTime(syncStats.next_scheduled_run)} />
  </div>
  
  <div class="action-buttons">
    <button 
      class="sync-button" 
      on:click={startSync} 
      disabled={isLoading}
    >
      <Fa icon={faSync} spin={isLoading} />
      {isLoading ? 'Synchronisiere...' : 'Jetzt synchronisieren'}
    </button>
  </div>
</div>

<style>
  .stat-panel {
    flex: 0.8;
    min-width: 125px;
    background-color: var(--mantle);
    border-radius: 6px;
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
  }

  .system-grid {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    flex: 1;
  }
  
  .error-message {
    margin: 0.5rem 0;
    padding: 0.5rem;
    background-color: rgba(230, 57, 70, 0.1);
    border-left: 3px solid var(--red);
    color: var(--red);
    font-size: 0.8rem;
  }
  
  .action-buttons {
    margin-top: 1rem;
    display: flex;
    gap: 0.5rem;
    justify-content: center;
    flex-wrap: wrap;
  }
  
  .sync-button {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    font-weight: 600;
    font-size: 0.8rem;
    cursor: pointer;
    transition: background-color 0.2s;
    background-color: var(--blue);
    color: var(--crust);
  }
  
  .sync-button:hover:not(:disabled) {
    background-color: var(--sapphire);
  }
  
  .sync-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>