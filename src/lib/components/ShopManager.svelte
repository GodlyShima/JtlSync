<script lang="ts">
  import { TauriApiService } from "$lib/services/TauriApiService";
  import type { AppConfig, ShopConfig } from "$lib/types";
  import { faCheck, faEdit, faPlus, faStore, faTrash } from "@fortawesome/free-solid-svg-icons";
  import { onMount } from "svelte";
  import Fa from "svelte-fa";
  import PanelHeader from "./stats/PanelHeader.svelte";
  
  // State variables
  let config: AppConfig;
  let isLoading = true;
  let error: string | null = null;
  let isAddingShop = false;
  let isEditingShop = false;
  let currentShopId: string = "";
  
  // New/edited shop form data
  let editingShop: ShopConfig = createEmptyShop();
  
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
  
  // Load the configuration when the component mounts
  onMount(async () => {
    await loadConfig();
  });
  
  // Load configuration from backend
  async function loadConfig() {
    try {
      isLoading = true;
      error = null;
      
      config = await TauriApiService.invoke<AppConfig>('load_config_command');
      
      // Set the current shop ID
      if (config.shops.length > 0) {
        currentShopId = config.shops[config.current_shop_index].id;
      }
      
    } catch (err) {
      console.error("Failed to load configuration:", err);
      error = String(err);
    } finally {
      isLoading = false;
    }
  }
  
  // Start adding a new shop
  function startAddShop() {
    editingShop = createEmptyShop();
    isAddingShop = true;
    isEditingShop = false;
  }
  
  // Start editing an existing shop
  function startEditShop(shop: ShopConfig) {
    // Clone the shop to avoid modifying the original
    editingShop = JSON.parse(JSON.stringify(shop));
    isEditingShop = true;
    isAddingShop = false;
  }
  
  // Save a new shop
  async function saveNewShop() {
    try {
      isLoading = true;
      error = null;
      
      // Validate shop data
      if (!editingShop.name) {
        error = "Shop name is required";
        return;
      }
      
      if (!editingShop.joomla.host || !editingShop.joomla.user || !editingShop.joomla.database) {
        error = "Joomla connection details are required";
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
        return;
      }
      
      if (!editingShop.joomla.host || !editingShop.joomla.user || !editingShop.joomla.database) {
        error = "Joomla connection details are required";
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
    if (!confirm(`Are you sure you want to delete this shop? This action cannot be undone.`)) {
      return;
    }
    
    try {
      isLoading = true;
      error = null;
      
      // Delete the shop
      config = await TauriApiService.invoke<AppConfig>('remove_shop_command', { shop_id: shopId });
      
    } catch (err) {
      console.error("Failed to delete shop:", err);
      error = String(err);
    } finally {
      isLoading = false;
    }
  }
  
  // Set the current active shop
  async function setCurrentShop(shopId: string) {
    try {
      isLoading = true;
      error = null;
      
      // Set the current shop
      config = await TauriApiService.invoke<AppConfig>('set_current_shop_command', { shop_id: shopId });
      
      // Update the current shop ID
      currentShopId = shopId;
      
    } catch (err) {
      console.error("Failed to set current shop:", err);
      error = String(err);
    } finally {
      isLoading = false;
    }
  }
  
  // Cancel the add/edit form
  function cancelForm() {
    isAddingShop = false;
    isEditingShop = false;
    editingShop = createEmptyShop();
  }
</script>

<div class="stat-panel">
  <PanelHeader icon={faStore} title="Shop Manager" />
  
  {#if error}
    <div class="error-message">{error}</div>
  {/if}
  
  <div class="shop-content">
    {#if isLoading && !config}
      <div class="loading">Loading shops...</div>
    {:else}
      <div class="shops-list">
        {#each config?.shops || [] as shop (shop.id)}
          <div class="shop-item {currentShopId === shop.id ? 'active' : ''}">
            <div class="shop-header">
              <div class="shop-name-section">
                {#if currentShopId === shop.id}
                  <div class="active-indicator">
                    <Fa icon={faCheck} size="sm" />
                  </div>
                {/if}
                <span class="shop-name">{shop.name}</span>
              </div>
              <div class="shop-actions">
                {#if currentShopId !== shop.id}
                  <button 
                    class="action-btn activate" 
                    on:click={() => setCurrentShop(shop.id)}
                    title="Set as Active Shop"
                  >
                    <Fa icon={faCheck} size="sm" />
                  </button>
                {/if}
                <button 
                  class="action-btn edit" 
                  on:click={() => startEditShop(shop)}
                  title="Edit Shop"
                >
                  <Fa icon={faEdit} size="sm" />
                </button>
                {#if config?.shops?.length > 1}
                  <button 
                    class="action-btn delete" 
                    on:click={() => deleteShop(shop.id)}
                    title="Delete Shop"
                  >
                    <Fa icon={faTrash} size="sm" />
                  </button>
                {/if}
              </div>
            </div>
            <div class="shop-details">
              <div class="shop-detail">Joomla DB: {shop.joomla.database}@{shop.joomla.host}</div>
              <div class="shop-detail">Tables: {shop.tables.orders}</div>
            </div>
          </div>
        {/each}
        
        {#if isAddingShop || isEditingShop}
          <div class="shop-form">
            <div class="form-header">
              <h4>{isAddingShop ? 'Add New Shop' : 'Edit Shop'}</h4>
            </div>
            
            <div class="form-section">
              <h5>General Information</h5>
              <div class="form-group">
                <label for="shop-name">Shop Name</label>
                <input 
                  type="text" 
                  id="shop-name" 
                  bind:value={editingShop.name}
                  placeholder="My Shop"
                  required
                />
              </div>
            </div>
            
            <div class="form-section">
              <h5>Joomla Database</h5>
              <div class="form-group">
                <label for="joomla-host">Host</label>
                <input 
                  type="text" 
                  id="joomla-host" 
                  bind:value={editingShop.joomla.host}
                  placeholder="localhost or domain"
                  required
                />
              </div>
              <div class="form-group">
                <label for="joomla-user">Username</label>
                <input 
                  type="text" 
                  id="joomla-user" 
                  bind:value={editingShop.joomla.user}
                  placeholder="Database username"
                  required
                />
              </div>
              <div class="form-group">
                <label for="joomla-password">Password</label>
                <input 
                  type="password" 
                  id="joomla-password" 
                  bind:value={editingShop.joomla.password}
                  placeholder="Database password"
                />
              </div>
              <div class="form-group">
                <label for="joomla-database">Database</label>
                <input 
                  type="text" 
                  id="joomla-database" 
                  bind:value={editingShop.joomla.database}
                  placeholder="Database name"
                  required
                />
              </div>
            </div>
            
            <div class="form-section">
              <h5>JTL Database</h5>
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
                <label for="jtl-user">Username</label>
                <input 
                  type="text" 
                  id="jtl-user" 
                  bind:value={editingShop.jtl.user}
                  placeholder="Database username"
                  required
                />
              </div>
              <div class="form-group">
                <label for="jtl-password">Password</label>
                <input 
                  type="password" 
                  id="jtl-password" 
                  bind:value={editingShop.jtl.password}
                  placeholder="Database password"
                />
              </div>
              <div class="form-group">
                <label for="jtl-database">Database</label>
                <input 
                  type="text" 
                  id="jtl-database" 
                  bind:value={editingShop.jtl.database}
                  placeholder="Database name"
                  required
                />
              </div>
            </div>
            
            <div class="form-section">
              <h5>VirtueMart Tables</h5>
              <div class="form-group">
                <label for="table-orders">Orders Table</label>
                <input 
                  type="text" 
                  id="table-orders" 
                  bind:value={editingShop.tables.orders}
                  placeholder="jos_virtuemart_orders"
                  required
                />
              </div>
              <div class="form-group">
                <label for="table-items">Order Items Table</label>
                <input 
                  type="text" 
                  id="table-items" 
                  bind:value={editingShop.tables.orderItems}
                  placeholder="jos_virtuemart_order_items"
                  required
                />
              </div>
              <div class="form-group">
                <label for="table-customers">Customers Table</label>
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
              <button class="cancel-btn" on:click={cancelForm}>Cancel</button>
              <button 
                class="save-btn" 
                on:click={isAddingShop ? saveNewShop : updateShop}
                disabled={isLoading}
              >
                {isLoading ? 'Saving...' : 'Save Shop'}
              </button>
            </div>
          </div>
        {:else}
          <button class="add-shop-btn" on:click={startAddShop}>
            <Fa icon={faPlus} size="sm" />
            Add New Shop
          </button>
        {/if}
      </div>
    {/if}
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

  .shop-content {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    flex: 1;
    overflow-y: auto;
    max-height: 70vh;
  }
  
  .error-message {
    margin: 0.5rem 0;
    padding: 0.5rem;
    background-color: rgba(230, 57, 70, 0.1);
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
  
  .shop-item {
    background-color: var(--surface0);
    border-radius: 6px;
    padding: 0.75rem;
    border-left: 3px solid var(--surface1);
    transition: all 0.2s ease;
  }
  
  .shop-item:hover {
    background-color: var(--surface1);
  }
  
  .shop-item.active {
    border-left-color: var(--blue);
    background-color: var(--surface1);
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
  
  .shop-details {
    font-size: 0.75rem;
    color: var(--subtext0);
  }
  
  .shop-detail {
    margin-bottom: 0.25rem;
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
  
  .shop-form {
    background-color: var(--surface0);
    border-radius: 6px;
    padding: 1rem;
    border-left: 3px solid var(--blue);
    margin-top: 1rem;
  }
  
  .form-header {
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