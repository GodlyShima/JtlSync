<script lang="ts">
  import { TauriApiService } from "$lib/services/TauriApiService";
  import type { LogEntry } from "$lib/types";
  import { faTerminal, faTrash } from "@fortawesome/free-solid-svg-icons";
  import { onDestroy, onMount } from "svelte";
  import Fa from "svelte-fa";
  import PanelHeader from "./PanelHeader.svelte";
  
  // State for logs
  let logs: LogEntry[] = [];
  let filterText = "";
  let categoryFilter: "all" | "sync" | "api" | "system" = "all";
  let levelFilter: "all" | "info" | "warn" | "error" | "debug" = "all";
  let autoScroll = true;
  
  // Reference to the log container for auto-scrolling
  let logContainer: HTMLDivElement;
  
  // Handle new log entries
  function handleNewLog(event: { payload: any }) {
    const log = event.payload as LogEntry;
    console.log("Received log:", log);
    
    // Convert timestamp to Date if it's a string (which might happen from JSON)
    const timestamp = typeof log.timestamp === 'string' 
      ? new Date(log.timestamp)
      : log.timestamp;
    
    const newLog: LogEntry = {
      timestamp,
      message: log.message,
      level: log.level,
      category: log.category
    };
    
    logs = [newLog, ...logs].slice(0, 1000); // Keep only the latest 1000 logs
    
    // Apply auto-scroll if enabled
    if (autoScroll && logContainer) {
      setTimeout(() => {
        logContainer.scrollTop = 0;
      }, 10);
    }
  }
  
  // Format timestamp
  function formatTime(date: Date): string {
    return new Intl.DateTimeFormat('de-DE', {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    }).format(date);
  }
  
  // Get log row color based on level
  function getLogColor(level: string): string {
    switch (level) {
      case "error": return "var(--red)";
      case "warn": return "var(--yellow)";
      case "info": return "var(--blue)";
      case "debug": return "var(--text)";
      default: return "var(--text)";
    }
  }
  
  // Filter logs based on criteria
  $: filteredLogs = logs.filter(log => {
    const matchesText = filterText === "" || 
      log.message.toLowerCase().includes(filterText.toLowerCase());
    
    const matchesCategory = categoryFilter === "all" || 
      log.category === categoryFilter;
    
    const matchesLevel = levelFilter === "all" || 
      log.level === levelFilter;
    
    return matchesText && matchesCategory && matchesLevel;
  });
  
  // Clear all logs
  function clearLogs() {
    logs = [];
  }
  
  // Toggle auto-scroll
  function toggleAutoScroll() {
    autoScroll = !autoScroll;
  }
  
  // Set up event listeners
  onMount(() => {
    console.log("LogPanel mounted, setting up listeners");
    
    // Listen for log events from the backend
    TauriApiService.listen("log", handleNewLog);
    
    // Add initial log
    handleNewLog({
      payload: {
        timestamp: new Date(),
        message: "Log panel initialized",
        level: "info",
        category: "system"
      }
    });
  });
  
  // Clean up
  onDestroy(() => {
    console.log("LogPanel unmounting, removing listeners");
    TauriApiService.unlisten("log", handleNewLog);
  });
</script>

<div class="stat-panel">
  <PanelHeader icon={faTerminal} title="Logs" />
  
  <div class="log-toolbar">
    <div class="filter-container">
      <input 
        type="text" 
        bind:value={filterText} 
        placeholder="Logs filtern..." 
        class="filter-input"
      />
      
      <select bind:value={categoryFilter} class="filter-select">
        <option value="all">Alle Kategorien</option>
        <option value="sync">Sync</option>
        <option value="api">API</option>
        <option value="system">System</option>
      </select>
      
      <select bind:value={levelFilter} class="filter-select">
        <option value="all">Alle Level</option>
        <option value="info">Info</option>
        <option value="warn">Warnung</option>
        <option value="error">Fehler</option>
        <option value="debug">Debug</option>
      </select>
    </div>
    
    <div class="log-actions">
      <button 
        class="action-button" 
        on:click={toggleAutoScroll} 
        title={autoScroll ? "Auto-Scroll deaktivieren" : "Auto-Scroll aktivieren"}
      >
        <div class={`auto-scroll-indicator ${autoScroll ? 'active' : ''}`}></div>
      </button>
      
      <button 
        class="action-button" 
        on:click={clearLogs} 
        title="Logs löschen"
      >
        <Fa icon={faTrash} size="sm" />
      </button>
    </div>
  </div>
  
  <div class="log-container" bind:this={logContainer}>
    {#if filteredLogs.length === 0}
      <div class="no-logs">Keine Logs gefunden</div>
    {:else}
      <table class="log-table">
        <tbody>
          {#each filteredLogs as log, i (i)}
            <tr class="log-row" data-level={log.level}>
              <td class="log-time">{formatTime(log.timestamp)}</td>
              <td class="log-category">{log.category}</td>
              <td class="log-level" style="color: {getLogColor(log.level)}">
                {log.level}
              </td>
              <td class="log-message">{log.message}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
  
  <div class="log-status">
    {filteredLogs.length} von {logs.length} Logs angezeigt
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
    height: 300px; /* Fixed height for the entire panel */
  }

  .log-toolbar {
    display: flex;
    justify-content: space-between;
    margin-bottom: 0.5rem;
    gap: 0.5rem;
    flex-shrink: 0; /* Prevent the toolbar from shrinking */
  }
  
  .filter-container {
    display: flex;
    gap: 0.5rem;
    flex: 1;
  }
  
  .filter-input {
    flex: 1;
    padding: 0.25rem 0.5rem;
    font-size: 0.75rem;
    border-radius: 4px;
    border: 1px solid var(--surface0);
    background-color: var(--surface0);
    color: var(--text);
  }
  
  .filter-select {
    padding: 0.25rem;
    font-size: 0.75rem;
    border-radius: 4px;
    border: 1px solid var(--surface0);
    background-color: var(--surface0);
    color: var(--text);
  }
  
  .log-actions {
    display: flex;
    gap: 0.25rem;
  }
  
  .action-button {
    padding: 0.25rem;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    background-color: var(--surface0);
    color: var(--text);
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .action-button:hover {
    background-color: var(--surface1);
  }
  
  .auto-scroll-indicator {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background-color: var(--overlay0);
  }
  
  .auto-scroll-indicator.active {
    background-color: var(--green);
  }
  
  .log-container {
    flex: 1; /* Take remaining space */
    height: 200px; /* Set a fixed height for the log container */
    min-height: 200px; /* Ensure minimum height */
    overflow-y: auto; /* Enable vertical scrolling */
    border-radius: 4px;
    background-color: var(--crust);
    margin-bottom: 0.5rem;
    border: 1px solid var(--surface0);
  }
  
.log-table {
  width: 100%;
  border-collapse: collapse;
  font-family: monospace;
  font-size: 0.75rem;
  table-layout: fixed; /* Keep fixed table layout */
}
  
.log-row {
  border-bottom: 1px solid var(--surface0);
  display: table-row; /* Ensure row behavior */
}
  
  .log-row:last-child {
    border-bottom: none;
  }
  
  .log-row:hover {
    background-color: var(--surface0);
  }
  
  .log-row[data-level="error"] {
    border-left: 3px solid var(--red);
  }
  
  .log-row[data-level="warn"] {
    border-left: 3px solid var(--yellow);
  }
  
  .log-row[data-level="info"] {
    border-left: 3px solid var(--blue);
  }
  
  .log-row[data-level="debug"] {
    border-left: 3px solid var(--overlay0);
  }
  
.log-time, .log-category, .log-level {
  padding: 0.25rem;
  white-space: nowrap;
  color: var(--subtext0);
  width: auto; /* Instead of 1% */
}

.log-time {
  width: 80px; /* Fixed width for timestamp */
}

.log-category {
  width: 70px; /* Fixed width for category */
}

.log-level {
  width: 60px; /* Fixed width for level */
}

.log-message {
  padding: 0.25rem;
  word-break: break-word;
  width: auto; /* Instead of 97% */
}
  
  .no-logs {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
    padding: 1rem;
    color: var(--subtext0);
    font-style: italic;
  }
  
  .log-status {
    font-size: 0.7rem;
    color: var(--subtext0);
    text-align: right;
    flex-shrink: 0; /* Prevent status from shrinking */
    margin-top: 0.25rem;
  }
</style>