// src/lib/services/SchedulerService.ts
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { get, writable } from "svelte/store";

interface ScheduledJob {
  id: string;
  name: string;
  cronExpression: string;
  lastRun: string | null;
  nextRun: string | null;
  enabled: boolean;
  shopIds: string[]; // Array of shop IDs to sync
}

export interface ScheduleStore {
  jobs: ScheduledJob[];
  isSchedulerRunning: boolean;
}

// Create a store for scheduled jobs
const createScheduleStore = () => {
  const initialState: ScheduleStore = {
    jobs: [],
    isSchedulerRunning: false,
  };

  const { subscribe, set, update } = writable<ScheduleStore>(initialState);

  return {
    subscribe,
    set,
    update,

    // Add a new scheduled job
    addJob: (job: Omit<ScheduledJob, "id">) => {
      const id = crypto.randomUUID();
      update((state) => ({
        ...state,
        jobs: [...state.jobs, { ...job, id }],
      }));
      return id;
    },

    // Update an existing job
    updateJob: (id: string, updates: Partial<ScheduledJob>) => {
      update((state) => ({
        ...state,
        jobs: state.jobs.map((job) =>
          job.id === id ? { ...job, ...updates } : job
        ),
      }));
    },

    // Remove a job by ID
    removeJob: (id: string) => {
      update((state) => ({
        ...state,
        jobs: state.jobs.filter((job) => job.id !== id),
      }));
    },

    // Enable/disable a specific job
    toggleJobEnabled: (id: string) => {
      update((state) => ({
        ...state,
        jobs: state.jobs.map((job) =>
          job.id === id ? { ...job, enabled: !job.enabled } : job
        ),
      }));
    },

    // Set scheduler running state
    setSchedulerRunning: (isRunning: boolean) => {
      update((state) => ({ ...state, isSchedulerRunning: isRunning }));
    },

    // Load jobs from localStorage
    loadJobs: async () => {
      try {
        const storedJobs = localStorage.getItem("jtlsync-scheduled-jobs");
        if (storedJobs) {
          const jobs = JSON.parse(storedJobs) as ScheduledJob[];
          update((state) => ({ ...state, jobs }));
        }
      } catch (err) {
        console.error("Failed to load scheduled jobs:", err);
      }
    },

    // Save jobs to localStorage
    saveJobs: async () => {
      try {
        const state = get({ subscribe });
        localStorage.setItem(
          "jtlsync-scheduled-jobs",
          JSON.stringify(state.jobs)
        );
      } catch (err) {
        console.error("Failed to save scheduled jobs:", err);
      }
    },

    // Update all next run times
    updateNextRunTimes: () => {
      update((state) => ({
        ...state,
        jobs: state.jobs.map((job) => {
          if (!job.enabled) return job;

          const nextRun = getNextRunFromCron(job.cronExpression);
          return {
            ...job,
            nextRun: nextRun ? nextRun.toISOString() : job.nextRun,
          };
        }),
      }));
    },
  };
};

export const scheduleStore = createScheduleStore();

// Parse cron expression to Date
function getNextRunFromCron(cronExpression: string): Date | null {
  // For simplicity, we'll just support a basic format: "hour:minute"
  try {
    const [hour, minute] = cronExpression.split(":").map(Number);

    if (
      isNaN(hour) ||
      isNaN(minute) ||
      hour < 0 ||
      hour > 23 ||
      minute < 0 ||
      minute > 59
    ) {
      return null;
    }

    const now = new Date();
    const next = new Date();
    next.setHours(hour, minute, 0, 0);

    // If the time has already passed today, schedule for tomorrow
    if (next <= now) {
      next.setDate(next.getDate() + 1);
    }

    return next;
  } catch (err) {
    console.error("Invalid cron expression:", err);
    return null;
  }
}

// Format next run time into a human-readable format
function formatNextRun(date: Date | null): string {
  if (!date) return "Unbekannt";

  const now = new Date();
  const diff = date.getTime() - now.getTime();

  // If negative, it's in the past
  if (diff < 0) {
    return "Fällig";
  }

  const minutes = Math.floor(diff / (1000 * 60));
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;

  if (hours === 0) {
    return `in ${remainingMinutes} Minuten`;
  } else if (hours < 24) {
    return `in ${hours} Stunden und ${remainingMinutes} Minuten`;
  } else {
    const days = Math.floor(hours / 24);
    return `in ${days} Tagen`;
  }
}

// Check if a job should run now
function shouldRunJob(job: ScheduledJob): boolean {
  if (!job.enabled) return false;

  const nextRun = job.nextRun ? new Date(job.nextRun) : null;
  if (!nextRun) return false;

  const now = new Date();

  // Debug log to see what's being compared
  console.log(
    `Job ${
      job.name
    } next run: ${nextRun.toISOString()}, now: ${now.toISOString()}`
  );

  // We consider it should run if the scheduled time is in the past
  return nextRun <= now;
}

// UI timer to update countdown displays
let uiUpdateInterval: number | null = null;

// The interval for the scheduler
let schedulerInterval: number | null = null;

// Start the scheduler
export async function startScheduler(): Promise<void> {
  if (get(scheduleStore).isSchedulerRunning) return;

  scheduleStore.setSchedulerRunning(true);
  await scheduleStore.loadJobs();

  // Update next run times for all jobs
  scheduleStore.updateNextRunTimes();

  // Save the updated jobs
  await scheduleStore.saveJobs();

  // Add a UI update interval (every 30 seconds) to keep the countdown fresh
  if (uiUpdateInterval) {
    clearInterval(uiUpdateInterval);
  }

  uiUpdateInterval = setInterval(() => {
    const { jobs } = get(scheduleStore);
    // This will trigger UI updates for the countdown displays
    scheduleStore.set({ jobs, isSchedulerRunning: true });
  }, 30000) as unknown as number;

  // Start the scheduler interval for checking jobs (every minute)
  if (schedulerInterval) {
    clearInterval(schedulerInterval);
  }

  schedulerInterval = setInterval(async () => {
    const { jobs, isSchedulerRunning } = get(scheduleStore);

    if (!isSchedulerRunning) {
      if (schedulerInterval) {
        clearInterval(schedulerInterval);
        schedulerInterval = null;
      }
      if (uiUpdateInterval) {
        clearInterval(uiUpdateInterval);
        uiUpdateInterval = null;
      }
      return;
    }

    // Check each job
    for (const job of jobs) {
      if (shouldRunJob(job)) {
        // Run the job
        console.log(`Running scheduled job: ${job.name}`);

        try {
          // Determine which shops to sync
          const shopIds = job.shopIds?.length > 0 ? job.shopIds : [];

          // Run the sync with the provided shop IDs - PARAMETER KORREKTUR HIER
          await invoke("start_scheduled_sync", {
            shop_ids: shopIds, // Geändert von shopIds zu shop_ids
            job_id: job.id, // Geändert von jobId zu job_id
          });

          // Update job's last run time
          const now = new Date();
          const nextRun = getNextRunFromCron(job.cronExpression);

          scheduleStore.updateJob(job.id, {
            lastRun: now.toISOString(),
            nextRun: nextRun ? nextRun.toISOString() : null,
          });

          await scheduleStore.saveJobs();

          // Show a notification
          const appWindow = await getCurrentWindow();
          await appWindow.emit("show-notification", {
            title: "Geplante Synchronisation gestartet",
            body: `Die geplante Synchronisation "${job.name}" wurde gestartet.`,
          });
        } catch (err) {
          console.error(`Failed to run scheduled job ${job.name}:`, err);

          // Show error notification via event
          const appWindow = await getCurrentWindow();
          await appWindow.emit("show-notification", {
            title: "Fehler bei der geplanten Synchronisation",
            body: `Die geplante Synchronisation "${job.name}" konnte nicht ausgeführt werden.`,
          });
        }
      }
    }
  }, 60000); // Check every minute
}

// Stop the scheduler
export function stopScheduler(): void {
  scheduleStore.setSchedulerRunning(false);
  if (schedulerInterval) {
    clearInterval(schedulerInterval);
    schedulerInterval = null;
  }
  if (uiUpdateInterval) {
    clearInterval(uiUpdateInterval);
    uiUpdateInterval = null;
  }
}

// Add a new scheduled job
export async function addScheduledJob(
  name: string,
  cronExpression: string,
  shopIds: string[] = []
): Promise<string> {
  const nextRun = getNextRunFromCron(cronExpression);

  const jobId = scheduleStore.addJob({
    name,
    cronExpression,
    lastRun: null,
    nextRun: nextRun ? nextRun.toISOString() : null,
    enabled: true,
    shopIds: shopIds,
  });

  await scheduleStore.saveJobs();
  return jobId;
}

// Human-readable format for when a job will next run
export function getNextRunText(job: ScheduledJob): string {
  if (!job.enabled) return "Deaktiviert";
  if (!job.nextRun) return "Nicht geplant";

  return formatNextRun(new Date(job.nextRun));
}
