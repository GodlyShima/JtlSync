<script lang="ts">
  import { TauriApiService } from "$lib/services/TauriApiService";
  import type { AppConfig, ShopConfig, SyncStats } from "$lib/types";
  import { faCheck, faClock, faEdit, faPlus, faStore, faSync, faTimes, faTrash } from "@fortawesome/free-solid-svg-icons";
  import { onMount } from "svelte";
  import Fa from "svelte-fa";
  import PanelHeader from "./stats/PanelHeader.svelte";
  
  // State variables
  let config: AppConfig;
  let activeShop: ShopConfig;
  let isLoading = true;
  let isSyncing = false;
  let error: string | null = null;
  
  // Edit/Add state
  let isAddingShop = false;
  let isEditingShop = false;
  let editingShop: ShopConfig = createEmptyShop();
  
  // Multi-sync state
  let selectedShopIds: string[] = [];
  let isSettingTimeframe = false;
  let currentTimeframe: number = 24; // Default to 24 hours
  let timeframeShopId: string = "";
  
  // Initialize sync stats
  let syncStats: any = {};
  
  // Create an empty shop object
  function createEmptyShop(): ShopConfig {
    return {
      id: generateId(),
      name: "",
      joomla: {
        host: "",
        user: "",
        password: "",
        database: ""
      },
      jtl: {
        host: "localhost",
        user: "root",
        password: "",
        database: "jtl"
      },
      tables: {
        orders: "jos_virtuemart_orders",
        orderItems: "jos_virtuemart_order_items",
        customers: "jos_virtuemart_order_userinfos"
      }
    };
  }
  
  // Generate a random ID for new shops
  function generateId(): string {
    return 'shop_' + Math.random().toString(36).substring(2, 9);
  }
  
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
  
  // Toggle shop selection for multi-sync
  function toggleShopSelection(shopId: string) {
    if (selectedShopIds.includes(shopId)) {
      selectedShopIds = selectedShopIds.filter(id => id !== shopId);
    } else {
      selectedShopIds = [...selectedShopIds, shopId];
    }
  }
  
  // Select all shops for multi-sync
  function selectAllShops() {
    if (config && config.shops) {
      selectedShopIds = config.shops.map(shop => shop.id);
    }
  }
  
  // Deselect all shops
  function deselectAllShops() {
    selectedShopIds = [];
  }
  
  // Load configuration
  async function loadConfig() {
    try {
      isLoading = true;
      error = null;
      
      config = await TauriApiService.invoke<AppConfig>('load_config_command');
      
      if (config.shops.length > 0) {
        activeShop = config.shops[config.current_shop_index];
        // Load stats for active shop
        await loadShopStats(activeShop.id);
      }
      
    } catch (err) {
      console.error("Failed to load configuration:", err);
      error = String(err);
    } finally {
      isLoading = false;
    }
  }
  
  // Load stats for a specific shop
  async function loadShopStats(shopId: string) {
    try {
      const stats = await TauriApiService.invoke('get_sync_stats', { shop_id: shopId });
      syncStats = { ...syncStats, [shopId]: stats };
    } catch (err) {
      console.warn("Failed to get shop stats, sync may not have run yet:", err);
    }
  }
  
  // Load stats for all shops
  async function loadAllShopStats() {
    if (!config || !config.shops) return;
    
    for (const shop of config.shops) {
      await loadShopStats(shop.id);
    }
  }
  
  // Set the current active shop
async function setCurrentShop(shopId: string) {
  try {
    isLoading = true;
    error = null;
    
		console.log(shopId);

    // Fix here: The parameter needs to be named shop_id to match backend expectations
    config = await TauriApiService.invoke<AppConfig>('set_current_shop_command', { 
      shop_id: shopId  // Changed from shopId to shop_id to match Rust backend
    });
    
    // Update active shop
    activeShop = config.shops.find(shop => shop.id === shopId) || config.shops[0];
    
    // Load stats for the newly active shop
    await loadShopStats(shopId);
    
  } catch (err) {
    console.error("Failed to set current shop:", err);
    error = String(err);
  } finally {
    isLoading = false;
  }
}
  
  // Start editing an existing shop
  function startEditShop(shop: ShopConfig) {
    // Clone the shop to avoid modifying the original
    editingShop = JSON.parse(JSON.stringify(shop));
    isEditingShop = true;
    isAddingShop = false;
  }
  
  // Start adding a new shop
  function startAddShop() {
    editingShop = createEmptyShop();
    isAddingShop = true;
    isEditingShop = false;
  }
  
  // Show timeframe configuration modal
  function showTimeframeConfig(shopId: string, currentHours: number) {
    timeframeShopId = shopId;
    currentTimeframe = currentHours;
    isSettingTimeframe = true;
  }
  
  // Save a new shop
  async function saveNewShop() {
    try {
      isLoading = true;
      error = null;
      
      // Validate shop data
      if (!editingShop.name) {
        error = "Shop name is required";
        isLoading = false;
        return;
      }
      
      if (!editingShop.joomla.host || !editingShop.joomla.user || !editingShop.joomla.database) {
        error = "Joomla connection details are required";
        isLoading = false;
        return;
      }
      
      // Add the new shop
      config = await TauriApiService.invoke<AppConfig>('add_shop_command', { shop: editingShop });
      
      // Reset form state
      isAddingShop = false;
      editingShop = createEmptyShop();
      
    } catch (err) {
      console.error("Failed to add shop:", err);
      error = String(err);
    } finally {
      isLoading = false;
    }
  }
  
  // Update an existing shop
  async function updateShop() {
    try {
      isLoading = true;
      error = null;
      
      // Validate shop data
      if (!editingShop.name) {
        error = "Shop name is required";
        isLoading = false;
        return;
      }
      
      if (!editingShop.joomla.host || !editingShop.joomla.user || !editingShop.joomla.database) {
        error = "Joomla connection details are required";
        isLoading = false;
        return;
      }
      
      // Update the shop
      config = await TauriApiService.invoke<AppConfig>('update_shop_command', { shop: editingShop });
      
      // Reset form state
      isEditingShop = false;
      editingShop = createEmptyShop();
      
    } catch (err) {
      console.error("Failed to update shop:", err);
      error = String(err);
    } finally {
      isLoading = false;
    }
  }
  
  // Delete a shop
  async function deleteShop(shopId: string) {
    if (!confirm(`Sind Sie sicher, dass Sie diesen Shop löschen möchten? Diese Aktion kann nicht rückgängig gemacht werden.`)) {
      return;
    }
    
    try {
      isLoading = true;
      error = null;
      
      // Delete the shop
      config = await TauriApiService.invoke<AppConfig>('remove_shop_command', { shop_id: shopId });
      
      // Update active shop if needed
      if (activeShop.id === shopId) {
        activeShop = config.shops[config.current_shop_index];
      }
      
    } catch (err) {
      console.error("Failed to delete shop:", err);
      error = String(err);
    } finally {
      isLoading = false;
    }
  }
  
  // Update timeframe for a shop
  async function updateTimeframe() {
    if (!timeframeShopId || currentTimeframe <= 0) {
      error = "Invalid timeframe configuration";
      return;
    }
    
    try {
      isLoading = true;
      error = null;
      
      // Update timeframe
      const stats = await TauriApiService.invoke<SyncStats>('set_sync_hours', { 
        shop_id: timeframeShopId, 
        hours: currentTimeframe 
      });
      
      // Update stats
      syncStats = { ...syncStats, [timeframeShopId]: stats };
      
      // Reset form state
      isSettingTimeframe = false;
      timeframeShopId = "";
      
    } catch (err) {
      console.error("Failed to update timeframe:", err);
      error = String(err);
    } finally {
      isLoading = false;
    }
  }
  
  // Cancel the edit/add form
  function cancelForm() {
    isAddingShop = false;
    isEditingShop = false;
    isSettingTimeframe = false;
    editingShop = createEmptyShop();
    timeframeShopId = "";
  }
  
  // Start sync for a specific shop
  async function startSync(shopId: string, hours?: number) {
    if (isSyncing) return;
    
    try {
      isSyncing = true;
      error = null;
      
      // Start sync for specific shop, optionally with custom timeframe
      await TauriApiService.invoke('start_sync_command', { 
        shop_id: shopId,
        hours: hours
      });
      
    } catch (err) {
      console.error("Failed to start sync:", err);
      error = String(err);
      isSyncing = false;
    }
  }
  
  // Start multi-shop sync
  async function startMultiSync() {
    if (isSyncing || selectedShopIds.length === 0) return;
    
    try {
      isSyncing = true;
      error = null;
      
      await TauriApiService.invoke('start_multi_sync_command', { shop_ids: selectedShopIds });
      
    } catch (err) {
      console.error("Failed to start multi-shop sync:", err);
      error = String(err);
      isSyncing = false;
    }
  }
  
  // Handle sync events for updates
  function handleSyncProgress(event: { payload: any }) {
    const [shopId, stats] = event.payload;
    syncStats = { ...syncStats, [shopId]: stats };
  }
  
  // Handle sync completion
  function handleSyncComplete(event: { payload: any }) {
    const [shopId, stats] = event.payload;
    syncStats = { ...syncStats, [shopId]: stats };
    isSyncing = false;
  }
  
  // Handle multi-sync completion
  function handleMultiSyncComplete() {
    loadAllShopStats();
    isSyncing = false;
  }
  
  // Handle sync error
  function handleSyncError(event: { payload: any }) {
    const payload = event.payload;
    
    if (Array.isArray(payload)) {
      const [err, shopId] = payload;
      error = `Error with shop ${shopId}: ${err}`;
    } else {
      error = `Sync error: ${payload}`;
    }
    
    isSyncing = false;
  }
  
  // Setup listeners and load config on mount
  onMount(() => {
    // Set up event listeners
    let unlistenProgress: (() => void) | null = null;
    let unlistenComplete: (() => void) | null = null;
    let unlistenMultiComplete: (() => void) | null = null;
    let unlistenError: (() => void) | null = null;
    
    // Register listeners asynchronously
    const setup = async () => {
      unlistenProgress = await TauriApiService.listen("sync-stats-update", handleSyncProgress);
      unlistenComplete = await TauriApiService.listen("sync-complete", handleSyncComplete);
      unlistenMultiComplete = await TauriApiService.listen("multi-sync-complete", handleMultiSyncComplete);
      unlistenError = await TauriApiService.listen("sync-error", handleSyncError);
      
      // Load initial configuration
      await loadConfig();
    };
    
    // Start the async setup
    setup();
    
    // Return cleanup function - this needs to be synchronous
    return () => {
      // Clean up listeners if they were initialized
      if (unlistenProgress) unlistenProgress();
      if (unlistenComplete) unlistenComplete(); 
      if (unlistenMultiComplete) unlistenMultiComplete();
      if (unlistenError) unlistenError();
    };
  });
</script>

<div class="shop-dashboard">
  <PanelHeader icon={faStore} title="Shop Manager" />
  
  {#if error}
    <div class="error-message">{error}</div>
  {/if}
  
  <div class="shop-content">
    {#if isLoading && !config}
      <div class="loading">Lädt Shops...</div>
    {:else if isAddingShop || isEditingShop}
      <div class="shop-form">
        <div class="form-header">
          <h4>{isAddingShop ? 'Neuen Shop hinzufügen' : 'Shop bearbeiten'}</h4>
          <button class="action-btn close" on:click={cancelForm} title="Abbrechen">
            <Fa icon={faTimes} />
          </button>
        </div>
        
        <div class="form-section">
          <h5>Allgemeine Informationen</h5>
          <div class="form-group">
            <label for="shop-name">Shop Name</label>
            <input 
              type="text" 
              id="shop-name" 
              bind:value={editingShop.name}
              placeholder="Mein Shop"
              required
            />
          </div>
        </div>
        
        <div class="form-section">
          <h5>Joomla Datenbank</h5>
          <div class="form-group">
            <label for="joomla-host">Host</label>
            <input 
              type="text" 
              id="joomla-host" 
              bind:value={editingShop.joomla.host}
              placeholder="localhost oder Domain"
              required
            />
          </div>
          <div class="form-group">
            <label for="joomla-user">Benutzername</label>
            <input 
              type="text" 
              id="joomla-user" 
              bind:value={editingShop.joomla.user}
              placeholder="Datenbank Benutzername"
              required
            />
          </div>
          <div class="form-group">
            <label for="joomla-password">Passwort</label>
            <input 
              type="password" 
              id="joomla-password" 
              bind:value={editingShop.joomla.password}
              placeholder="Datenbank Passwort"
            />
          </div>
          <div class="form-group">
            <label for="joomla-database">Datenbank</label>
            <input 
              type="text" 
              id="joomla-database" 
              bind:value={editingShop.joomla.database}
              placeholder="Datenbankname"
              required
            />
          </div>
        </div>
        
        <div class="form-section">
          <h5>JTL Datenbank</h5>
          <div class="form-group">
            <label for="jtl-host">Host</label>
            <input 
              type="text" 
              id="jtl-host" 
              bind:value={editingShop.jtl.host}
              placeholder="localhost"
              required
            />
          </div>
          <div class="form-group">
            <label for="jtl-user">Benutzername</label>
            <input 
              type="text" 
              id="jtl-user" 
              bind:value={editingShop.jtl.user}
              placeholder="Datenbank Benutzername"
              required
            />
          </div>
          <div class="form-group">
            <label for="jtl-password">Passwort</label>
            <input 
              type="password" 
              id="jtl-password" 
              bind:value={editingShop.jtl.password}
              placeholder="Datenbank Passwort"
            />
          </div>
          <div class="form-group">
            <label for="jtl-database">Datenbank</label>
            <input 
              type="text" 
              id="jtl-database" 
              bind:value={editingShop.jtl.database}
              placeholder="Datenbankname"
              required
            />
          </div>
        </div>
        
        <div class="form-section">
          <h5>VirtueMart Tabellen</h5>
          <div class="form-group">
            <label for="table-orders">Bestellungen Tabelle</label>
            <input 
              type="text" 
              id="table-orders" 
              bind:value={editingShop.tables.orders}
              placeholder="jos_virtuemart_orders"
              required
            />
          </div>
          <div class="form-group">
            <label for="table-items">Bestellpositionen Tabelle</label>
            <input 
              type="text" 
              id="table-items" 
              bind:value={editingShop.tables.orderItems}
              placeholder="jos_virtuemart_order_items"
              required
            />
          </div>
          <div class="form-group">
            <label for="table-customers">Kunden Tabelle</label>
            <input 
              type="text" 
              id="table-customers" 
              bind:value={editingShop.tables.customers}
              placeholder="jos_virtuemart_order_userinfos"
              required
            />
          </div>
        </div>
        
        <div class="form-actions">
          <button class="cancel-btn" on:click={cancelForm}>Abbrechen</button>
          <button 
            class="save-btn" 
            on:click={isAddingShop ? saveNewShop : updateShop}
            disabled={isLoading}
          >
            {isLoading ? 'Speichert...' : 'Shop speichern'}
          </button>
        </div>
      </div>
    {:else if isSettingTimeframe}
      <div class="timeframe-form">
        <div class="form-header">
          <h4>Zeitfenster konfigurieren</h4>
          <button class="action-btn close" on:click={cancelForm} title="Abbrechen">
            <Fa icon={faTimes} />
          </button>
        </div>
        
        <div class="form-group">
          <label for="timeframe">Zeitfenster (Stunden)</label>
          <input 
            type="number" 
            id="timeframe" 
            bind:value={currentTimeframe}
            min="1" 
            max="168"
            required
          />
          <small>Bestellungen aus den letzten X Stunden abrufen</small>
        </div>
        
        <div class="form-actions">
          <button class="cancel-btn" on:click={cancelForm}>Abbrechen</button>
          <button 
            class="save-btn" 
            on:click={updateTimeframe}
            disabled={isLoading || currentTimeframe <= 0}
          >
            {isLoading ? 'Speichert...' : 'Änderungen speichern'}
          </button>
        </div>
      </div>
    {:else}
      <!-- Multi-shop sync controls -->
      {#if config?.shops?.length > 1}
        <div class="multi-sync-controls">
          <div class="selection-controls">
            <button class="small-btn" on:click={selectAllShops}>Alle auswählen</button>
            <button class="small-btn" on:click={deselectAllShops}>Keine auswählen</button>
          </div>
          
          <button 
            class="multi-sync-button {selectedShopIds.length === 0 ? 'disabled' : ''}" 
            on:click={startMultiSync}
            disabled={isSyncing || selectedShopIds.length === 0}
          >
            <Fa icon={faSync} spin={isSyncing} />
            {isSyncing ? 'Synchronisiere...' : `${selectedShopIds.length} Shops synchronisieren`}
          </button>
        </div>
      {/if}
      
      <div class="shops-list">
        {#each config?.shops || [] as shop (shop.id)}
          <div class="shop-item {shop.id === activeShop?.id ? 'active' : ''}">
            <!-- Shop selection checkbox for multi-sync -->
            <div class="shop-select">
              <input 
                type="checkbox" 
                id="select-{shop.id}" 
                checked={selectedShopIds.includes(shop.id)}
                on:change={() => toggleShopSelection(shop.id)}
              />
            </div>
            
            <div class="shop-content-wrapper">
              <div class="shop-header">
                <div class="shop-name-section">
                  {#if shop.id === activeShop?.id}
                    <div class="active-indicator">
                      <Fa icon={faCheck} size="sm" />
                    </div>
                  {/if}
                  <span class="shop-name">{shop.name}</span>
                </div>
                
                <div class="shop-actions">
                  {#if shop.id !== activeShop?.id}
                    <button 
                      class="action-btn activate" 
                      on:click={() => setCurrentShop(shop.id)}
                      title="Als aktiven Shop setzen"
                    >
                      <Fa icon={faCheck} size="sm" />
                    </button>
                  {/if}
                  <button 
                    class="action-btn edit" 
                    on:click={() => startEditShop(shop)}
                    title="Shop bearbeiten"
                  >
                    <Fa icon={faEdit} size="sm" />
                  </button>
                  <button 
                    class="action-btn timeframe" 
                    on:click={() => showTimeframeConfig(shop.id, syncStats[shop.id]?.sync_hours || 24)}
                    title="Synchronisationszeitraum einstellen"
                  >
                    <Fa icon={faClock} size="sm" />
                  </button>
                  {#if config?.shops?.length > 1}
                    <button 
                      class="action-btn delete" 
                      on:click={() => deleteShop(shop.id)}
                      title="Shop löschen"
                    >
                      <Fa icon={faTrash} size="sm" />
                    </button>
                  {/if}
                </div>
              </div>
              
              <!-- Shop details -->
              <div class="shop-details">
                <div class="shop-detail">Joomla DB: {shop.joomla.database}@{shop.joomla.host}</div>
                
                <!-- Shop stats if available -->
                {#if syncStats[shop.id]}
                  <div class="shop-stats">
                    <div class="stat-row">
                      <span class="stat-label">Zeitfenster:</span>
                      <span class="stat-value">{syncStats[shop.id].sync_hours} Stunden</span>
                    </div>
                    {#if syncStats[shop.id].last_sync_time}
                      <div class="stat-row">
                        <span class="stat-label">Letzte Synchronisation:</span>
                        <span class="stat-value">{formatDateTime(new Date(syncStats[shop.id].last_sync_time))}</span>
                      </div>
                    {/if}
                    <div class="stat-row">
                      <span class="stat-label">Erfolgreich:</span>
                      <span class="stat-value">{syncStats[shop.id].synced_orders}</span>
                    </div>
                    <div class="stat-row">
                      <span class="stat-label">Übersprungen:</span>
                      <span class="stat-value">{syncStats[shop.id].skipped_orders}</span>
                    </div>
                  </div>
                {/if}
              </div>
              
              <div class="shop-actions-row">
                <button 
                  class="sync-button" 
                  on:click={() => startSync(shop.id)}
                  disabled={isSyncing}
                >
                  <Fa icon={faSync} spin={isSyncing && selectedShopIds.includes(shop.id)} />
                  {isSyncing && selectedShopIds.includes(shop.id) ? 'Synchronisiere...' : 'Jetzt synchronisieren'}
                </button>
              </div>
            </div>
          </div>
        {/each}
        
        <button class="add-shop-btn" on:click={startAddShop}>
          <Fa icon={faPlus} size="sm" />
          Neuen Shop hinzufügen
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .shop-dashboard {
    display: flex;
    flex-direction: column;
    height: 100%;
  }
  
  .shop-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    overflow-y: auto;
    padding-right: 0.25rem;
    padding: 15px;
  }
  
  .error-message {
    margin: 0.5rem 0;
    padding: 1rem;
    background-color: rgba(243, 139, 168, 0.1);
    border-left: 3px solid var(--red);
    color: var(--red);
    font-size: 0.8rem;
  }
  
  .loading {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    color: var(--subtext0);
    font-style: italic;
  }
  
  .shops-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .multi-sync-controls {
    background-color: var(--surface0);
    border-radius: 6px;
    padding: 0.75rem;
    margin-bottom: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  
  .selection-controls {
    display: flex;
    gap: 0.5rem;
  }
  
  .small-btn {
    background-color: var(--surface1);
    color: var(--text);
    border: none;
    border-radius: 0.3rem;
    padding: 0.25rem 0.5rem;
    font-size: 0.7rem;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .small-btn:hover {
    background-color: var(--surface2);
  }
  
  .multi-sync-button {
    background-color: var(--blue);
    color: var(--crust);
    border: none;
    border-radius: 0.3rem;
    padding: 0.5rem 1rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    font-weight: 500;
    transition: background-color 0.2s ease;
  }
  
  .multi-sync-button:hover:not(:disabled) {
    background-color: var(--sapphire);
  }
  
  .multi-sync-button.disabled,
  .multi-sync-button:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }
  
  .shop-item {
    background-color: var(--surface0);
    border-radius: 6px;
    padding: 0.75rem;
    border-left: 3px solid var(--surface1);
    transition: all 0.2s ease;
    display: flex;
    gap: 0.75rem;
  }
  
  .shop-item:hover {
    background-color: var(--surface1);
  }
  
  .shop-item.active {
    border-left-color: var(--blue);
    background-color: var(--surface1);
  }
  
  .shop-select {
    display: flex;
    align-items: center;
  }
  
  .shop-content-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
  }
  
  .shop-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }
  
  .shop-name-section {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .active-indicator {
    color: var(--blue);
    display: flex;
    align-items: center;
  }
  
  .shop-name {
    font-weight: 600;
    font-size: 0.9rem;
  }
  
  .shop-actions {
    display: flex;
    gap: 0.5rem;
  }
  
  .action-btn {
    background: none;
    border: none;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--subtext0);
    width: 24px;
    height: 24px;
    border-radius: 4px;
    transition: background-color 0.2s, color 0.2s;
    padding: 0;
  }
  
  .action-btn:hover {
    background-color: var(--surface2);
    color: var(--text);
  }
  
  .action-btn.activate {
    color: var(--green);
  }
  
  .action-btn.edit {
    color: var(--blue);
  }
  
  .action-btn.delete {
    color: var(--red);
  }
  
  .action-btn.close {
    color: var(--subtext0);
  }
  
  .action-btn.timeframe {
    color: var(--yellow);
  }
  
  .shop-details {
    font-size: 0.75rem;
    color: var(--subtext0);
  }
  
  .shop-detail {
    margin-bottom: 0.25rem;
  }
  
  .shop-stats {
    background-color: var(--surface0);
    border-radius: 0.3rem;
    padding: 0.5rem;
    margin: 0.5rem 0;
    font-size: 0.8rem;
  }
  
  .stat-row {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.2rem;
  }
  
  .stat-label {
    color: var(--subtext0);
  }
  
  .stat-value {
    font-weight: 500;
  }
  
  .shop-actions-row {
    display: flex;
    margin-top: 0.5rem;
  }
  
  .add-shop-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    background-color: var(--surface1);
    color: var(--text);
    border: 1px dashed var(--surface2);
    border-radius: 6px;
    padding: 0.75rem;
    cursor: pointer;
    transition: all 0.2s ease;
    margin-top: 0.5rem;
  }
  
  .add-shop-btn:hover {
    background-color: var(--surface2);
    border-color: var(--blue);
    color: var(--blue);
  }
  
  .sync-button {
    background-color: var(--blue);
    color: var(--crust);
    border: none;
    border-radius: 0.3rem;
    padding: 0.5rem 1rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    font-weight: 500;
    width: 100%;
    transition: background-color 0.2s ease;
  }
  
  .sync-button:hover:not(:disabled) {
    background-color: var(--sapphire);
  }
  
  .sync-button:disabled {
    opacity: 0.7;
    cursor: not-allowed;
  }
  
  .shop-form, .timeframe-form {
    background-color: var(--surface0);
    border-radius: 6px;
    padding: 1rem;
    border-left: 3px solid var(--blue);
  }
  
  .form-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }
  
  .form-header h4 {
    font-size: 1rem;
    font-weight: 600;
    margin: 0;
    color: var(--blue);
  }
  
  .form-section {
    margin-bottom: 1.5rem;
    border-top: 1px solid var(--surface1);
    padding-top: 1rem;
  }
  
  .form-section h5 {
    font-size: 0.85rem;
    font-weight: 600;
    margin: 0 0 0.75rem 0;
    color: var(--subtext1);
  }
  
  .form-group {
    margin-bottom: 0.75rem;
  }
  
  .form-group label {
    display: block;
    font-size: 0.75rem;
    margin-bottom: 0.25rem;
    color: var(--subtext0);
  }
  
  .form-group input {
    width: 100%;
    padding: 0.5rem;
    border-radius: 4px;
    border: 1px solid var(--surface1);
    background-color: var(--surface0);
    color: var(--text);
    font-size: 0.8rem;
    transition: border-color 0.2s;
  }
  
  .form-group input:focus {
    outline: none;
    border-color: var(--blue);
  }
  
  .form-group input::placeholder {
    color: var(--overlay0);
  }
  
  .form-group small {
    display: block;
    font-size: 0.7rem;
    color: var(--subtext0);
    margin-top: 0.25rem;
  }
  
  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 1.5rem;
  }
  
  .cancel-btn {
    padding: 0.5rem 1rem;
    border-radius: 4px;
    background-color: var(--surface1);
    color: var(--text);
    border: none;
    font-size: 0.8rem;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .cancel-btn:hover {
    background-color: var(--surface2);
  }
  
  .save-btn {
    padding: 0.5rem 1rem;
    border-radius: 4px;
    background-color: var(--blue);
    color: var(--crust);
    border: none;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .save-btn:hover:not(:disabled) {
    background-color: var(--sapphire);
  }
  
  .save-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>