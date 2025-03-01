import { writable } from "svelte/store";

interface ProcessStore {
  error: string | null;
  isLoading: boolean;
  searchTerm: string;
  currentPage: number;
}

const initialState: ProcessStore = {
  error: null,
  isLoading: true,
  searchTerm: "",
  currentPage: 1,
};

function createProcessStore() {
  const { subscribe, set, update } = writable<ProcessStore>(initialState);

  // Define all methods first
  const setIsLoading = (isLoading: boolean) =>
    update((state) => ({ ...state, isLoading }));

  const setSearchTerm = (searchTerm: string) =>
    update((state) => ({ ...state, searchTerm, currentPage: 1 }));

  const setIsFrozen = (isFrozen: boolean) =>
    update((state) => ({ ...state, isFrozen }));

  const setCurrentPage = (currentPage: number) =>
    update((state) => ({ ...state, currentPage }));

  // Return all methods
  return {
    subscribe,
    set,
    update,
    setIsLoading,
    setSearchTerm,
    setIsFrozen,
    setCurrentPage,
  };
}

export const processStore = createProcessStore();
