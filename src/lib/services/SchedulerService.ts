// src/lib/services/SchedulerService.ts
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { get, writable } from "svelte/store";

// Define schedule types
export enum ScheduleType {
  DAILY = "daily", // Run once per day at specific time
  HOURLY = "hourly", // Run every hour
  MINUTES = "minutes", // Run every X minutes
  CUSTOM = "custom", // Custom schedule
}

interface ScheduledJob {
  id: string;
  name: string;
  scheduleType: ScheduleType;
  cronExpression: string; // For DAILY: "HH:MM", for HOURLY: empty, for MINUTES: number of minutes
  interval?: number; // For MINUTES: interval in minutes
  lastRun: string | null;
  nextRun: string | null;
  enabled: boolean;
  shop_ids: string[]; // Changed from shopIds to shop_ids
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

          const nextRun = getNextRunTime(job);
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

// Calculate the next run time based on schedule type
function getNextRunTime(job: ScheduledJob): Date | null {
  const now = new Date();

  switch (job.scheduleType) {
    case ScheduleType.DAILY:
      return getNextRunFromDailyTime(job.cronExpression);

    case ScheduleType.HOURLY:
      // Run at the beginning of the next hour
      const nextHour = new Date(now);
      nextHour.setHours(nextHour.getHours() + 1, 0, 0, 0);
      return nextHour;

    case ScheduleType.MINUTES:
      // Run after the specified interval in minutes
      if (!job.interval || job.interval <= 0) return null;

      const nextInterval = new Date(now);
      nextInterval.setMinutes(nextInterval.getMinutes() + job.interval);
      return nextInterval;

    case ScheduleType.CUSTOM:
      // For custom schedule, we still support the old format
      return getNextRunFromDailyTime(job.cronExpression);

    default:
      return null;
  }
}

// Parse daily time format (HH:MM) to Date
function getNextRunFromDailyTime(timeString: string): Date | null {
  // For simple format: "hour:minute"
  try {
    const [hour, minute] = timeString.split(":").map(Number);

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
    console.error("Invalid time format:", err);
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

async function startScheduledSync(
  shopIds: string[],
  jobId: string
): Promise<void> {
  return invoke("start_scheduled_sync", {
    shop_ids: shopIds,
    job_id: jobId,
    shopIds: shopIds, // For compatibility
    jobId: jobId,
  });
}

// Check if a job should run now
function shouldRunJob(job: ScheduledJob): boolean {
  if (!job.enabled) return false;

  switch (job.scheduleType) {
    case ScheduleType.DAILY:
    case ScheduleType.CUSTOM:
      // For daily and custom schedules, we check the next run time
      const nextRun = job.nextRun ? new Date(job.nextRun) : null;
      if (!nextRun) return false;
      return nextRun <= new Date();

    case ScheduleType.HOURLY:
      // For hourly jobs, check if we're at the start of an hour
      const now = new Date();
      return now.getMinutes() === 0 && now.getSeconds() < 60;

    case ScheduleType.MINUTES:
      // For minute-based jobs, check if enough time has passed since last run
      if (!job.interval || job.interval <= 0) return false;
      if (!job.lastRun) return true; // Never run before

      const lastRun = new Date(job.lastRun);
      const minutesSinceLastRun =
        (Date.now() - lastRun.getTime()) / (1000 * 60);
      return minutesSinceLastRun >= job.interval;

    default:
      return false;
  }
}

// Get a descriptive text of the job schedule
export function getScheduleDescription(job: ScheduledJob): string {
  switch (job.scheduleType) {
    case ScheduleType.DAILY:
      return `Täglich um ${job.cronExpression} Uhr`;
    case ScheduleType.HOURLY:
      return "Stündlich";
    case ScheduleType.MINUTES:
      return `Alle ${job.interval} Minuten`;
    case ScheduleType.CUSTOM:
      return `Benutzerdefiniert: ${job.cronExpression}`;
    default:
      return "Unbekannter Zeitplan";
  }
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
        console.log(
          `Running scheduled job: ${job.name} (${getScheduleDescription(job)})`
        );

        try {
          // Determine which shops to sync
          const shopIds = job.shop_ids?.length > 0 ? job.shop_ids : [];

          try {
            // Pass directly with snake_case parameter names
            await startScheduledSync(job.shop_ids, job.id);
            console.log("Scheduled sync started successfully");
          } catch (err) {
            console.error("Failed to start scheduled sync:", err);
          }

          // Update job's last run time
          const now = new Date();

          // Calculate next run time based on schedule type
          const nextRun = getNextRunTime(job);

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
  scheduleType: ScheduleType,
  scheduleValue: string,
  interval?: number,
  shopIds: string[] = []
): Promise<string> {
  let cronExpression = scheduleValue;

  // Build a job with the correct schedule details
  const jobId = scheduleStore.addJob({
    name,
    scheduleType,
    cronExpression,
    interval: interval,
    lastRun: null,
    nextRun: null, // Will be calculated when the scheduler starts
    enabled: true,
    shop_ids: shopIds, // Use snake_case key here
  });

  // Update next run time for the new job
  scheduleStore.updateNextRunTimes();
  await scheduleStore.saveJobs();
  return jobId;
}

// Human-readable format for when a job will next run
export function getNextRunText(job: ScheduledJob): string {
  if (!job.enabled) return "Deaktiviert";

  // For recurring jobs without a fixed next time, show a different message
  if (
    job.scheduleType === ScheduleType.MINUTES &&
    (!job.nextRun || !job.lastRun)
  ) {
    return `Läuft alle ${job.interval} Minuten`;
  }

  if (job.scheduleType === ScheduleType.HOURLY && !job.nextRun) {
    return "Läuft stündlich";
  }

  if (!job.nextRun) return "Nicht geplant";

  return formatNextRun(new Date(job.nextRun));
}
