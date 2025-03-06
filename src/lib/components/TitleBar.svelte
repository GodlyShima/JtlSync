<script lang="ts">
  import { TauriApiService } from "$lib/services/TauriApiService";
  import { faCog, faMoon, faQuestionCircle, faSun, faSync } from "@fortawesome/free-solid-svg-icons";
  import { onDestroy, onMount } from "svelte";
  import Fa from "svelte-fa";
  
  // Application version
  let appVersion = "1.0.0";
  // Theme state
  let isDarkMode = true;
  // Show dropdown menu
  let showMenu = false;
  let showInfoPanel = false;
  
  // Get system info
  async function getSystemInfo() {
    try {
      const systemInfo = await TauriApiService.invoke("get_system_info");
      console.log("System info:", systemInfo);
    } catch (error) {
      console.error("Failed to get system info:", error);
    }
  }
  
  // Toggle dark/light mode
  function toggleTheme() {
    isDarkMode = !isDarkMode;
    document.documentElement.setAttribute('data-theme', isDarkMode ? 'dark' : 'light');
    
    // Store preference
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('theme', isDarkMode ? 'dark' : 'light');
    }
  }
  
  // Toggle dropdown menu
  function toggleMenu() {
    showMenu = !showMenu;
  }
  
  // Toggle info panel
  function toggleInfoPanel() {
    showInfoPanel = !showInfoPanel;
  }
  
  // Hide dropdown menu when clicking outside
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    const menu = document.querySelector('.menu-dropdown');
    const menuButton = document.querySelector('.menu-button');
    
    if (menu && !menu.contains(target) && menuButton && !menuButton.contains(target)) {
      showMenu = false;
    }
    
    // Also hide info panel if open
    const infoPanel = document.querySelector('.info-panel');
    const infoButton = document.querySelector('.info-button');
    
    if (infoPanel && !infoPanel.contains(target) && infoButton && !infoButton.contains(target)) {
      showInfoPanel = false;
    }
  }
  
  // Start sync process from title bar
  async function startQuickSync() {
    try {
      await TauriApiService.invoke('start_sync_command');
      // Add some visual feedback
      const syncButton = document.querySelector('.sync-button');
      if (syncButton) {
        syncButton.classList.add('syncing');
        setTimeout(() => {
          syncButton.classList.remove('syncing');
        }, 2000);
      }
    } catch (error) {
      console.error("Failed to start sync:", error);
    }
  }
  
  // Detect system theme preference on mount
  onMount(() => {
    // Add click event listener for hiding dropdown
    document.addEventListener('click', handleClickOutside);
    
    // Try to get system info
    getSystemInfo();
    
    // Check for saved theme preference
    if (typeof localStorage !== 'undefined') {
      const savedTheme = localStorage.getItem('theme');
      if (savedTheme) {
        isDarkMode = savedTheme === 'dark';
        document.documentElement.setAttribute('data-theme', savedTheme);
      } else {
        // Check system preference
        const prefersDarkMode = window.matchMedia('(prefers-color-scheme: dark)').matches;
        isDarkMode = prefersDarkMode;
        document.documentElement.setAttribute('data-theme', prefersDarkMode ? 'dark' : 'light');
      }
    }
  });
  
  // Clean up event listener on destroy
  onDestroy(() => {
    document.removeEventListener('click', handleClickOutside);
  });
</script>

<div class="title-bar" data-tauri-drag-region>
  <div class="title-section" data-tauri-drag-region>
    <img src="/32x32.png" alt="JTLSync" class="app-icon" />
    <div class="title-text">Kistenkolli JTLSync</div>
  </div>
  
  <div class="controls-section">
    <button class="title-button sync-button" on:click={startQuickSync} title="Schnell-Synchronisation starten">
      <Fa icon={faSync} />
    </button>
    
    <button class="title-button theme-button" on:click={toggleTheme} title="Design wechseln">
      <Fa icon={isDarkMode ? faSun : faMoon} />
    </button>
    
    <button class="title-button info-button" on:click={toggleInfoPanel} title="Info">
      <Fa icon={faQuestionCircle} />
    </button>
    
    <button class="title-button menu-button" on:click={toggleMenu} title="Menü">
      <Fa icon={faCog} />
    </button>
    
    {#if showMenu}
      <div class="menu-dropdown">
        <div class="menu-header">
          <span>Kistenkolli JTLSync</span>
          <span class="version">v{appVersion}</span>
        </div>
        
        <div class="menu-items">
          <button class="menu-item">
            <span class="item-icon">📁</span>
            <span class="item-text">Einstellungen</span>
          </button>
          
          <button class="menu-item">
            <span class="item-icon">💾</span>
            <span class="item-text">Datenbank</span>
          </button>
          
          <button class="menu-item">
            <span class="item-icon">🔄</span>
            <span class="item-text">Cache leeren</span>
          </button>
          
          <button class="menu-item">
            <span class="item-icon">📋</span>
            <span class="item-text">Log-Dateien</span>
          </button>
          
          <div class="menu-divider"></div>
          
          <button class="menu-item">
            <span class="item-icon">📱</span>
            <span class="item-text">Hilfe & Support</span>
          </button>
          
          <button class="menu-item">
            <span class="item-icon">ℹ️</span>
            <span class="item-text">Über JTLSync</span>
          </button>
        </div>
      </div>
    {/if}
    
    {#if showInfoPanel}
      <div class="info-panel">
        <h3>Kistenkolli JTLSync</h3>
        <p>Version: {appVersion}</p>
        <p>Entwickelt von Shima</p>
        <p>© 2025 Alle Rechte vorbehalten</p>
        
        <div class="info-actions">
          <button class="info-button">Dokumentation</button>
          <button class="info-button">Lizenz</button>
        </div>
      </div>
    {/if}
  </div>
</div>

<style>
  .title-bar {
    height: 38px;
    background: var(--surface0);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 0.75rem;
    -webkit-user-select: none;
    user-select: none;
    border-bottom: 1px solid var(--surface1);
    position: relative;
    z-index: 100;
  }

  .title-section {
    display: flex;
    align-items: center;
    height: 100%;
    gap: 0.5rem;
  }

  .app-icon {
    width: 20px;
    height: 20px;
  }

  .title-text {
    font-family: "Segoe UI", "SF Pro Display", -apple-system, BlinkMacSystemFont, sans-serif;
    font-size: 14px;
    font-weight: 500;
    color: var(--text);
    letter-spacing: 0.2px;
  }
  
  .controls-section {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    position: relative;
  }
  
  .title-button {
    width: 30px;
    height: 30px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: none;
    color: var(--text);
    border-radius: var(--border-radius-sm);
    cursor: pointer;
    transition: background-color var(--transition-fast);
  }
  
  .title-button:hover {
    background-color: var(--surface1);
  }
  
  .sync-button.syncing {
    animation: spin 1s linear infinite;
  }
  
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  
  .theme-button {
    color: var(--yellow);
  }
  
  .info-button {
    color: var(--blue);
  }
  
  .menu-dropdown {
    position: absolute;
    top: 40px;
    right: 0;
    background-color: var(--mantle);
    border-radius: var(--border-radius-md);
    box-shadow: var(--shadow-lg);
    width: 220px;
    z-index: 100;
    border: 1px solid var(--surface1);
    animation: fadeInDown 0.2s ease;
  }
  
  @keyframes fadeInDown {
    from { opacity: 0; transform: translateY(-10px); }
    to { opacity: 1; transform: translateY(0); }
  }
  
  .menu-header {
    padding: 0.75rem;
    border-bottom: 1px solid var(--surface0);
    font-size: 0.9rem;
    font-weight: 500;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .version {
    font-size: 0.75rem;
    color: var(--subtext0);
    font-weight: normal;
  }
  
  .menu-items {
    padding: 0.5rem;
  }
  
  .menu-item {
    width: 100%;
    text-align: left;
    padding: 0.5rem 0.75rem;
    border-radius: var(--border-radius-sm);
    background: none;
    border: none;
    color: var(--text);
    font-size: 0.85rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    transition: background-color var(--transition-fast);
  }
  
  .menu-item:hover {
    background-color: var(--surface1);
  }
  
  .item-icon {
    font-size: 1rem;
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 20px;
  }
  
  .menu-divider {
    height: 1px;
    background-color: var(--surface0);
    margin: 0.5rem 0;
  }
  
  .info-panel {
    position: absolute;
    top: 40px;
    right: 0;
    background-color: var(--mantle);
    border-radius: var(--border-radius-md);
    box-shadow: var(--shadow-lg);
    width: 300px;
    z-index: 100;
    border: 1px solid var(--surface1);
    padding: 1rem;
    animation: fadeInDown 0.2s ease;
  }
  
  .info-panel h3 {
    margin: 0 0 0.75rem 0;
    font-size: 1rem;
    color: var(--blue);
  }
  
  .info-panel p {
    margin: 0.5rem 0;
    font-size: 0.85rem;
    color: var(--text);
  }
  
  .info-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
  }
  
  .info-actions .info-button {
    padding: 0.4rem 0.75rem;
    background-color: var(--surface1);
    border-radius: var(--border-radius-sm);
    border: none;
    color: var(--text);
    font-size: 0.8rem;
    cursor: pointer;
    transition: background-color var(--transition-fast);
    width: auto;
    height: auto;
  }
  
  .info-actions .info-button:hover {
    background-color: var(--surface2);
  }
  
  /* Responsive styles */
  @media (max-width: 576px) {
    .title-text {
      font-size: 12px;
    }
    
    .app-icon {
      width: 16px;
      height: 16px;
    }
    
    .title-button {
      width: 28px;
      height: 28px;
    }
    
    .menu-dropdown, .info-panel {
      width: calc(100vw - 20px);
      right: -10px;
    }
  }
</style>