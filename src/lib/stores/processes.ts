import { DEFAULT_CONFIG } from "$lib/definitions/settings";
import { writable } from "svelte/store";

interface ProcessStore {
  error: string | null;
  isLoading: boolean;
  searchTerm: string;
  currentPage: number;
  itemsPerPage: number;
  isFrozen: boolean;
  refreshRate: number;
}

const initialState: ProcessStore = {
  error: null,
  isLoading: false,
  searchTerm: "",
  currentPage: 1,
  itemsPerPage: DEFAULT_CONFIG.behavior.itemsPerPage,
  isFrozen: false,
  refreshRate: DEFAULT_CONFIG.behavior.refreshRate,
};

function createProcessStore() {
  const { subscribe, set, update } = writable<ProcessStore>(initialState);

  return {
    subscribe,
    set,
    update,

    // Set loading state
    setIsLoading: (isLoading: boolean) =>
      update((state) => ({ ...state, isLoading })),

    // Set search term and reset page
    setSearchTerm: (searchTerm: string) =>
      update((state) => ({
        ...state,
        searchTerm,
        currentPage: 1,
      })),

    // Toggle frozen state for updates
    toggleFrozen: () =>
      update((state) => ({
        ...state,
        isFrozen: !state.isFrozen,
      })),

    // Set current page
    setCurrentPage: (currentPage: number) =>
      update((state) => ({ ...state, currentPage })),

    // Set items per page
    setItemsPerPage: (itemsPerPage: number) =>
      update((state) => ({
        ...state,
        itemsPerPage,
        currentPage: 1,
      })),

    // Set refresh rate
    setRefreshRate: (refreshRate: number) =>
      update((state) => ({ ...state, refreshRate })),

    // Set error message
    setError: (error: string | null) =>
      update((state) => ({ ...state, error })),

    // Reset to initial state
    reset: () => set(initialState),
  };
}

export const processStore = createProcessStore();
