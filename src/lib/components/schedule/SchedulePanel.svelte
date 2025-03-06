<!-- src/lib/components/schedule/SchedulePanel.svelte -->
<script lang="ts">
  import {
    addScheduledJob,
    getNextRunText,
    getScheduleDescription,
    scheduleStore,
    ScheduleType,
    startScheduler,
    stopScheduler
  } from '$lib/services/SchedulerService';
  import { TauriApiService } from "$lib/services/TauriApiService";
  import type { AppConfig } from "$lib/types";
  import { faCalendarAlt, faPause, faPlay, faSync, faTimes, faTrash } from '@fortawesome/free-solid-svg-icons';
  import { onDestroy, onMount } from 'svelte';
  import Fa from 'svelte-fa';
  import PanelHeader from '../stats/PanelHeader.svelte';
  
  let newJobName = '';
  let isAddingJob = false;
  let error: string | null = null;
  let config: AppConfig;
  let selectedShopIds: string[] = [];
  
  // Schedule settings
  let selectedScheduleType: ScheduleType = ScheduleType.DAILY;
  let dailyTime = '01:00'; // Default to 1:00 AM for daily schedule
  let minutesInterval = 30; // Default to 30 minutes for minutes-based schedule
  
  // For manual refresh functionality
  let lastUpdated = new Date();
  
  // For cleanup of intervals
  let refreshInterval: number | null = null;
  
  // Initialize on mount
  onMount(() => {
    // Setup refresh interval for UI updates
    refreshInterval = setInterval(() => {
      lastUpdated = new Date();
    }, 30000) as unknown as number;
    
    // Initialize async - separate from onMount return
    initializeScheduler();
    
    // Return the cleanup function (must be synchronous)
    return () => {
      if (refreshInterval) {
        clearInterval(refreshInterval);
        refreshInterval = null;
      }
    };
  });
  
  // Async initialization function
  async function initializeScheduler() {
    try {
      // Load config to get shops
      config = await TauriApiService.invoke<AppConfig>('load_config_command');
      
      // Start the scheduler
      await startScheduler();
    } catch (err) {
      console.error('Failed to initialize schedule panel:', err);
      error = `Failed to initialize: ${err}`;
    }
  }
  
  // Stop the scheduler on component destroy
  onDestroy(() => {
    stopScheduler();
  });
  
  // Toggle shop selection for job
  function toggleShopSelection(shopId: string) {
    if (selectedShopIds.includes(shopId)) {
      selectedShopIds = selectedShopIds.filter(id => id !== shopId);
    } else {
      selectedShopIds = [...selectedShopIds, shopId];
    }
  }
  
  // Add a new scheduled job
  async function handleAddJob() {
    if (!newJobName.trim()) {
      error = 'Bitte geben Sie einen Namen für den Job ein';
      return;
    }
    
    try {
      error = null;
      
      switch (selectedScheduleType) {
        case ScheduleType.DAILY:
          await addScheduledJob(newJobName, ScheduleType.DAILY, dailyTime, undefined, selectedShopIds);
          break;
          
        case ScheduleType.HOURLY:
          await addScheduledJob(newJobName, ScheduleType.HOURLY, "", undefined, selectedShopIds);
          break;
          
        case ScheduleType.MINUTES:
          if (minutesInterval < 1) {
            error = 'Das Intervall muss mindestens 1 Minute betragen';
            return;
          }
          await addScheduledJob(newJobName, ScheduleType.MINUTES, "", minutesInterval, selectedShopIds);
          break;
          
        default:
          error = 'Ungültiger Zeitplantyp';
          return;
      }
      
      newJobName = '';
      selectedShopIds = [];
      isAddingJob = false;
    } catch (err) {
      console.error('Failed to add scheduled job:', err);
      error = `Failed to add scheduled job: ${err}`;
    }
  }
  
  // Enable or disable a job
  async function toggleJobEnabled(id: string) {
    try {
      scheduleStore.toggleJobEnabled(id);
      await scheduleStore.saveJobs();
    } catch (err) {
      console.error('Failed to toggle job status:', err);
      error = `Failed to toggle job status: ${err}`;
    }
  }
  
  // Delete a scheduled job
  async function deleteJob(id: string) {
    if (!confirm('Sind Sie sicher, dass Sie diesen geplanten Job löschen möchten?')) {
      return;
    }
    
    try {
      scheduleStore.removeJob(id);
      await scheduleStore.saveJobs();
    } catch (err) {
      console.error('Failed to delete job:', err);
      error = `Failed to delete job: ${err}`;
    }
  }
  
  // Manual refresh function
  function refreshCountdowns() {
    lastUpdated = new Date();
  }
</script>

<div class="stat-panel">
  <PanelHeader icon={faCalendarAlt} title="Geplante Synchronisierung" />
  
  {#if error}
    <div class="error-message">{error}</div>
  {/if}
  
  <div class="schedule-content">
    {#if $scheduleStore.jobs.length === 0 && !isAddingJob}
      <div class="no-jobs">
        <p>Keine geplanten Aufgaben vorhanden</p>
        <button class="add-job-btn" on:click={() => isAddingJob = true}>
          Geplante Synchronisierung hinzufügen
        </button>
      </div>
    {:else}
      <div class="jobs-list">
        {#each $scheduleStore.jobs as job (job.id)}
          <div class="job-item {job.enabled ? '' : 'disabled'}">
            <div class="job-details">
              <div class="job-header">
                <span class="job-name">{job.name}</span>
                <div class="job-actions">
                  <button 
                    class="action-btn {job.enabled ? 'pause' : 'play'}" 
                    on:click={() => toggleJobEnabled(job.id)}
                    title={job.enabled ? 'Deaktivieren' : 'Aktivieren'}
                  >
                    <Fa icon={job.enabled ? faPause : faPlay} size="sm" />
                  </button>
                  <button 
                    class="action-btn delete" 
                    on:click={() => deleteJob(job.id)}
                    title="Löschen"
                  >
                    <Fa icon={faTrash} size="sm" />
                  </button>
                </div>
              </div>
              
              <!-- Schedule info based on type -->
              <div class="job-time">{getScheduleDescription(job)}</div>
              
              <!-- Using lastUpdated as a dependency to re-render countdown -->
              <div class="job-next-run">
                Nächste Ausführung: {getNextRunText(job)} 
                <!-- Adding lastUpdated dependency invisibly for reactivity -->
                {#if lastUpdated}<span class="hidden">{lastUpdated.toISOString()}</span>{/if}
              </div>
              
              {#if job.lastRun}
                <div class="job-last-run">
                  Letzte Ausführung: {new Date(job.lastRun).toLocaleString('de-DE')}
                </div>
              {/if}
              
              <!-- Shop selection display -->
              {#if job.shop_ids && job.shop_ids.length > 0 && config?.shops}
                <div class="job-shops">
                  <span>Shops: </span>
                  <div class="shop-tags">
                    {#each job.shop_ids as shopId}
                      {#if config.shops.find(s => s.id === shopId)}
                        <span class="shop-tag">{config.shops.find(s => s.id === shopId)?.name}</span>
                      {/if}
                    {/each}
                  </div>
                </div>
              {:else if config?.shops}
                <div class="job-shops">
                  <span>Shops: Alle</span>
                </div>
              {/if}
            </div>
          </div>
        {/each}
        
        {#if isAddingJob}
          <div class="add-job-form">
            <div class="form-header">
              <h4>Neue geplante Synchronisierung</h4>
              <button class="close-btn" on:click={() => isAddingJob = false}>
                <Fa icon={faTimes} />
              </button>
            </div>
            
            <div class="form-group">
              <label for="job-name">Name</label>
              <input 
                type="text" 
                id="job-name" 
                bind:value={newJobName}
                placeholder="z.B. Tägliche Synchronisierung"
              />
            </div>
            
            <!-- Schedule type selection -->
            <div class="form-group">
              <label>Zeitplan-Typ</label>
              <div class="radio-group">
                <label class="radio-label">
                  <input 
                    type="radio" 
                    name="schedule-type"
                    bind:group={selectedScheduleType} 
                    value={ScheduleType.DAILY}
                  />
                  Täglich
                </label>
                
                <label class="radio-label">
                  <input 
                    type="radio" 
                    name="schedule-type"
                    bind:group={selectedScheduleType} 
                    value={ScheduleType.HOURLY}
                  />
                  Stündlich
                </label>
                
                <label class="radio-label">
                  <input 
                    type="radio" 
                    name="schedule-type"
                    bind:group={selectedScheduleType} 
                    value={ScheduleType.MINUTES}
                  />
                  Minuten-Intervall
                </label>
              </div>
            </div>
            
            <!-- Fields specific to the selected schedule type -->
            {#if selectedScheduleType === ScheduleType.DAILY}
              <div class="form-group">
                <label for="job-time">Uhrzeit</label>
                <input 
                  type="time" 
                  id="job-time" 
                  bind:value={dailyTime}
                />
              </div>
            {:else if selectedScheduleType === ScheduleType.MINUTES}
              <div class="form-group">
                <label for="minutes-interval">Intervall (Minuten)</label>
                <input 
                  type="number" 
                  id="minutes-interval" 
                  bind:value={minutesInterval}
                  min="1"
                  max="1440"
                  step="1"
                />
                <small>Synchronisation alle {minutesInterval} Minuten</small>
              </div>
            {:else if selectedScheduleType === ScheduleType.HOURLY}
              <div class="form-info">
                Die Synchronisation wird zu Beginn jeder Stunde ausgeführt.
              </div>
            {/if}
            
            <!-- Shop selection -->
            {#if config?.shops}
              <div class="form-group">
                <label>Shops für diese Synchronisierung</label>
                <div class="shop-selection">
                  {#each config.shops as shop}
                    <div class="shop-selection-item">
                      <input 
                        type="checkbox" 
                        id="shop-{shop.id}" 
                        checked={selectedShopIds.includes(shop.id)}
                        on:change={() => toggleShopSelection(shop.id)}
                      />
                      <label for="shop-{shop.id}">{shop.name}</label>
                    </div>
                  {/each}
                </div>
                <small>Keine Auswahl bedeutet: Alle Shops synchronisieren</small>
              </div>
            {/if}
            
            <div class="form-actions">
              <button class="cancel-btn" on:click={() => isAddingJob = false}>Abbrechen</button>
              <button class="save-btn" on:click={handleAddJob}>Speichern</button>
            </div>
          </div>
        {:else}
          <div class="button-group">
            <button class="add-job-btn" on:click={() => isAddingJob = true}>
              Weitere Synchronisierung hinzufügen
            </button>
            <button class="refresh-btn" on:click={refreshCountdowns} title="Zeitanzeige aktualisieren">
              <Fa icon={faSync} size="sm" />
            </button>
          </div>
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

  .schedule-content {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
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
  
  .no-jobs {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 1rem;
    color: var(--subtext0);
    font-style: italic;
    gap: 1rem;
  }
  
  .jobs-list {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    overflow-y: auto;
    max-height: 250px;
  }
  
  .job-item {
    background-color: var(--surface0);
    border-radius: 6px;
    padding: 0.75rem;
    border-left: 3px solid var(--blue);
  }
  
  .job-item.disabled {
    border-left-color: var(--surface1);
    opacity: 0.7;
  }
  
  .job-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }
  
  .job-name {
    font-weight: 600;
    font-size: 0.9rem;
  }
  
  .job-actions {
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
    color: var(--text);
    width: 24px;
    height: 24px;
    border-radius: 4px;
    transition: background-color 0.2s;
  }
  
  .action-btn:hover {
    background-color: var(--surface1);
  }
  
  .action-btn.play {
    color: var(--green);
  }
  
  .action-btn.pause {
    color: var(--yellow);
  }
  
  .action-btn.delete {
    color: var(--red);
  }
  
  .job-time, .job-next-run, .job-last-run, .job-shops {
    font-size: 0.8rem;
    color: var(--subtext0);
    margin-bottom: 0.25rem;
  }
  
  .job-shops {
    margin-top: 0.5rem;
  }
  
  .shop-tags {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    margin-top: 0.25rem;
  }
  
  .shop-tag {
    background-color: var(--surface1);
    color: var(--text);
    font-size: 0.7rem;
    padding: 0.1rem 0.4rem;
    border-radius: 0.2rem;
  }
  
  .shop-selection {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-top: 0.5rem;
    max-height: 120px;
    overflow-y: auto;
    background-color: var(--surface1);
    padding: 0.5rem;
    border-radius: 0.3rem;
  }
  
  .shop-selection-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .shop-selection-item label {
    font-size: 0.8rem;
    cursor: pointer;
  }
  
  .hidden {
    display: none;
  }
  
  .button-group {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 0.5rem;
  }
  
  .add-job-btn {
    background-color: var(--blue);
    color: var(--crust);
    border: none;
    border-radius: 4px;
    padding: 0.5rem 1rem;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .add-job-btn:hover {
    background-color: var(--sapphire);
  }
  
  .refresh-btn {
    background-color: var(--surface1);
    color: var(--text);
    border: none;
    border-radius: 4px;
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .refresh-btn:hover {
    background-color: var(--surface2);
  }
  
  .add-job-form {
    background-color: var(--surface0);
    border-radius: 6px;
    padding: 1rem;
    border-left: 3px solid var(--lavender);
    margin-top: 0.5rem;
  }
  
  .form-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }
  
  .form-header h4 {
    margin: 0;
    font-size: 0.9rem;
    font-weight: 600;
  }
  
  .close-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: var(--subtext0);
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    transition: background-color 0.2s;
  }
  
  .close-btn:hover {
    background-color: var(--surface1);
    color: var(--text);
  }
  
  .form-group {
    margin-bottom: 0.75rem;
  }
  
  .form-group label {
    display: block;
    font-size: 0.8rem;
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
  }
  
  .form-group input:focus {
    outline: none;
    border-color: var(--blue);
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
    gap: 0.5rem;
    margin-top: 1rem;
  }
  
  .cancel-btn {
    background-color: var(--surface1);
    color: var(--text);
    border: none;
    border-radius: 4px;
    padding: 0.5rem 1rem;
    font-size: 0.8rem;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .cancel-btn:hover {
    background-color: var(--surface2);
  }
  
  .save-btn {
    background-color: var(--blue);
    color: var(--crust);
    border: none;
    border-radius: 4px;
    padding: 0.5rem 1rem;
    font-size: 0.8rem;
    font-weight: 600;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .save-btn:hover {
    background-color: var(--sapphire);
  }
</style>