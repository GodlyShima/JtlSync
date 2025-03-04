<!-- src/lib/components/schedule/SchedulePanel.svelte -->
<script lang="ts">
  import { addScheduledJob, getNextRunText, scheduleStore, startScheduler, stopScheduler } from '$lib/services/SchedulerService';
  import { faCalendarAlt, faPause, faPlay, faSync, faTimes, faTrash } from '@fortawesome/free-solid-svg-icons';
  import { onDestroy, onMount } from 'svelte';
  import Fa from 'svelte-fa';
  import PanelHeader from '../stats/PanelHeader.svelte';
  
  let newJobName = '';
  let newJobTime = '01:00'; // Default to 1:00 AM
  let isAddingJob = false;
  let error: string | null = null;
  
  // For manual refresh functionality
  let lastUpdated = new Date();
  
  // Start the scheduler on mount
  onMount(() => {
    startScheduler().catch(err => {
      console.error('Failed to start scheduler:', err);
      error = `Failed to start scheduler: ${err}`;
    });
    
    // Add an interval to refresh the "lastUpdated" time every 30 seconds
    // This will force Svelte to re-evaluate getNextRunText for all jobs
    const refreshInterval = setInterval(() => {
      lastUpdated = new Date();
    }, 30000);
    
    return () => {
      clearInterval(refreshInterval);
    };
  });
  
  // Stop the scheduler on component destroy
  onDestroy(() => {
    stopScheduler();
  });
  
  // Add a new scheduled job
  async function handleAddJob() {
    if (!newJobName.trim()) {
      error = 'Bitte geben Sie einen Namen für den Job ein';
      return;
    }
    
    try {
      error = null;
      await addScheduledJob(newJobName, newJobTime);
      newJobName = '';
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
              <div class="job-time">Zeit: {job.cronExpression}</div>
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
            <div class="form-group">
              <label for="job-time">Uhrzeit</label>
              <input 
                type="time" 
                id="job-time" 
                bind:value={newJobTime}
              />
            </div>
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
  
  .job-time, .job-next-run, .job-last-run {
    font-size: 0.8rem;
    color: var(--subtext0);
    margin-bottom: 0.25rem;
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