// Database configuration interface
export interface DatabaseConfig {
  host: string;
  user: string;
  password: string;
  database: string;
}

export interface Column {
  id: keyof VirtueMartOrder;
  label: string;
  visible: boolean;
  required?: boolean;
  format?: (value: any) => string;
}

// Tables configuration interface
export interface TablesConfig {
  orders: string;
  orderItems: string;
  customers: string;
}
// Application configuration
export interface AppConfig {
  joomla: DatabaseConfig;
  jtl: DatabaseConfig;
  tables: TablesConfig;
  logFile: string;
  jtlApiPath: string;
}

export interface ToolConfig {
  behavior: {
    itemsPerPage: number;
    refreshRate: number;
    defaultStatusFilter: string;
  };
}

// Synchronization statistics
export interface SyncStats {
  totalOrders: number;
  syncedOrders: number;
  skippedOrders: number;
  errorOrders: number;
  lastSyncTime: Date | null;
  nextScheduledRun: Date | null;
}

// JTL Order Item interface
export interface JtlOrderItem {
  Quantity: number;
  SalesPriceGross: number;
  TaxRate: number;
  Name: string;
  SalesUnit: string;
  SalesPriceNet: number;
  PurchasePriceNet?: number;
}

// VirtueMart Order interface from the database
export interface VirtueMartOrder {
  virtuemart_order_id: number;
  virtuemart_user_id: number;
  created_on: string;
  order_total: number;
  virtuemart_paymentmethod_id: number;
  order_status: string;
  virtuemart_shipmentmethod_id: string;
  order_number: string;
  coupon_code?: string;
  coupon_discount?: number;
  order_shipment?: number;
  order_shipment_tax?: number;
  customer_note?: string;
  first_name: string;
  last_name: string;
  company?: string;
  address_1: string;
  address_2?: string;
  zip: string;
  city: string;
  country_2_code: string;
  email: string;
  phone_1?: string;
  phone_2?: string;
  fax?: string;
  title?: string;
  virtuemart_order_userinfo_id: number;
}

// Log entry interface for displaying logs
export interface LogEntry {
  timestamp: Date;
  message: string;
  level: "info" | "warn" | "error" | "debug";
  category: "sync" | "api" | "system";
}

// System information interface
export interface SystemInfo {
  platform: string;
  arch: string;
  memory: string;
  uptime: number;
  nodeVersion?: string;
  currentDirectory?: string;
}

// Theme
export interface Theme {
  name: string;
  label: string;
  colors: {
    base: string;
    mantle: string;
    crust: string;
    text: string;
    subtext0: string;
    subtext1: string;
    surface0: string;
    surface1: string;
    surface2: string;
    overlay0: string;
    overlay1: string;
    blue: string;
    lavender: string;
    sapphire: string;
    sky: string;
    red: string;
    maroon: string;
    peach: string;
    yellow: string;
    green: string;
    teal: string;
  };
}
