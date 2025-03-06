<script lang="ts">
  import OrdersTable from "$lib/components/orders/OrdersTable.svelte";
  import ShopDashboard from "$lib/components/ShopDashboard.svelte";
  import { StatsBar } from "$lib/components/stats";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import ToolBar from "$lib/components/toolbar/ToolBar.svelte";
  import { column_definitions } from "$lib/definitions";
  import { TauriApiService } from "$lib/services/TauriApiService";
  import { processStore } from "$lib/stores/processes";
  import type { AppConfig, VirtueMartOrder } from "$lib/types";
  import { onMount } from "svelte";

  // Reactive store state destructuring
  $: ({
    error,
    searchTerm,
    isLoading,
    currentPage,
    itemsPerPage,
    isFrozen,
    refreshRate
  } = $processStore);

  // Columns configuration
  $: columns = column_definitions.map((col) => ({
    ...col,
    visible: col.required || col.visible
  }));

  // Orders state
  let orders: VirtueMartOrder[] = [];
  let activeShopId: string | null = null;
  let config: AppConfig | null = null;

  // Fetch orders from backend
  async function fetchSyncedOrders(shopId?: string) {
    try {
      processStore.setIsLoading(true);
      
      // Get synced orders for specific shop or all shops
      const fetchedOrders = await TauriApiService.invoke<VirtueMartOrder[]>(
        'get_synced_orders',
        shopId ? { shop_id: shopId } : {}
      );
      
      orders = fetchedOrders || [];
    } catch (err) {
      processStore.setError(String(err));
    } finally {
      processStore.setIsLoading(false);
    }
  }

  // Load configuration to get active shop
  async function loadConfig() {
    try {
      config = await TauriApiService.invoke<AppConfig>('load_config_command');
      
      if (config && config.shops.length > 0) {
        // Set active shop ID
        activeShopId = config.shops[config.current_shop_index].id;
        
        // Fetch orders for active shop
        await fetchSyncedOrders(activeShopId);
      }
    } catch (err) {
      console.error("Failed to load config:", err);
      processStore.setError(String(err));
    }
  }

  // Handle when new orders are synced
  function handleSyncedOrders(event: { payload: any }) {
    // The payload can be [shopId, orders] tuple or just orders for all shops
    if (Array.isArray(event.payload) && typeof event.payload[0] === 'string') {
      const [shopId, shopOrders] = event.payload;
      
      // Only update orders if this is for the active shop or we're showing all shops
      if (!activeShopId || shopId === activeShopId) {
        orders = shopOrders;
      }
    } else {
      // All synced orders
      orders = event.payload;
    }
  }

  // Set up event listeners
  onMount(() => {
    // Load config and get active shop
    loadConfig();
    
    // Listen for synced orders updates
    TauriApiService.listen('synced-orders', handleSyncedOrders);
    TauriApiService.listen('synced-orders-all', handleSyncedOrders);
    
    // Clean up event listeners
    return () => {
      TauriApiService.unlisten('synced-orders', handleSyncedOrders);
      TauriApiService.unlisten('synced-orders-all', handleSyncedOrders);
    };
  });

  // Filter orders based on the active shop
  $: filteredOrders = activeShopId 
    ? orders.filter(order => order.shop_id === activeShopId)
    : orders;

  // Calculate pagination
  $: totalPages = Math.ceil(filteredOrders.length / itemsPerPage);
</script>

<div class="app-container">
  <TitleBar />
  <main>
    <div class="dashboard-layout">
      <div class="main-content">
        <StatsBar />
        
        <ToolBar 
          bind:searchTerm 
          bind:itemsPerPage
          bind:currentPage
          bind:refreshRate
          bind:isFrozen
          totalPages={totalPages}
          totalResults={filteredOrders.length}
        />

        {#if error}
          <div class="alert error">{error}</div>
        {/if}

        <div class="orders-section">
          <OrdersTable columns={columns} orders={filteredOrders} />
        </div>
      </div>
      
      <div class="sidebar">
        <ShopDashboard />
      </div>
    </div>
  </main>
</div>

<style>
  /* Root variables are now in app.css */
  
  main {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: min-content;
    overflow: hidden;
  }

  .app-container {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background-color: var(--base);
    color: var(--text);
    transition: all 0.3s ease;
  }

  .dashboard-layout {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    padding: 0.5rem;
  }
  
  .orders-section {
    flex: 1;
    display: flex;
    flex-direction: column;
    border-radius: 6px;
    background-color: var(--mantle);
    overflow: hidden;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
    margin-top: 0.5rem;
  }

  .sidebar {
    width: 340px;
    border-left: 1px solid var(--surface0);
    background-color: var(--base);
    overflow-y: auto;
    transition: width 0.3s ease;
  }

  .alert.error {
    background-color: rgba(var(--red-rgb), 0.2);
    color: var(--red);
    border-left: 4px solid var(--red);
    padding: 10px;
    margin: 0.5rem 0;
    border-radius: var(--border-radius-sm);
    font-size: 0.9rem;
  }
  
  /* Responsive styles */
  @media (max-width: 1200px) {
    .sidebar {
      width: 300px;
    }
  }
  
  @media (max-width: 992px) {
    .dashboard-layout {
      flex-direction: column;
    }
    
    .sidebar {
      width: 100%;
      max-height: 400px;
      border-left: none;
      border-top: 1px solid var(--surface0);
    }
  }
  
  @media (max-width: 768px) {
    .sidebar {
      max-height: 350px;
    }
  }
  
  @media (max-width: 576px) {
    .main-content {
      padding: 0.25rem;
    }
  }
</style>