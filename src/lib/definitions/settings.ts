import type { ToolConfig } from "$lib/types";

export const DEFAULT_CONFIG: ToolConfig = {
  behavior: {
    itemsPerPage: 15,
    refreshRate: 1000,
    defaultStatusFilter: "all",
  },
};
