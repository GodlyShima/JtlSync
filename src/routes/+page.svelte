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

        <OrdersTable columns={columns} orders={filteredOrders} />
      </div>
      
      <div class="sidebar">
        <ShopDashboard />
      </div>
    </div>
  </main>
</div>

<style>
  :global(:root) {
    --base: #1e1e2e;
    --mantle: #181825;
    --crust: #11111b;
    --text: #cdd6f4;
    --subtext0: #a6adc8;
    --subtext1: #bac2de;
    --surface0: #313244;
    --surface1: #45475a;
    --surface2: #585b70;
    --overlay0: #6c7086;
    --overlay1: #7f849c;
    --blue: #89b4fa;
    --lavender: #b4befe;
    --sapphire: #74c7ec;
    --sky: #89dceb;
    --red: #f38ba8;
    --maroon: #eba0ac;
    --peach: #fab387;
    --yellow: #f9e2af;
    --green: #a6e3a1;
    --teal: #94e2d5;
    
    /* RGB values for transparency uses */
    --red-rgb: 243, 139, 168;
    --green-rgb: 166, 227, 161;
    --blue-rgb: 137, 180, 250;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial,
      sans-serif, "Apple Color Emoji", "Segoe UI Emoji";
    background-color: var(--base);
    color: var(--text);
    -webkit-font-smoothing: antialiased;
    overflow: hidden;
    user-select: none;
  }

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
  }

  .sidebar {
    width: 300px;
    border-left: 1px solid var(--surface0);
    background-color: var(--mantle);
    overflow-y: auto;
  }

  .alert.error {
    background-color: var(--red);
    color: var(--base);
    padding: 10px;
    text-align: center;
  }
</style>