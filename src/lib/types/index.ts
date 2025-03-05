// Database configuration interface
export interface DatabaseConfig {
  host: string;
  user: string;
  password: string;
  database: string;
}

// Tables configuration interface
export interface TablesConfig {
  orders: string;
  orderItems: string;
  customers: string;
}

// Shop configuration
export interface ShopConfig {
  id: string;
  name: string;
  joomla: DatabaseConfig;
  jtl: DatabaseConfig;
  tables: TablesConfig;
}

// Application configuration
export interface AppConfig {
  shops: ShopConfig[];
  current_shop_index: number;
  logFile: string;
  jtlApiPath: string;
}

// Table column configuration
export interface Column {
  id: keyof VirtueMartOrder;
  label: string;
  visible: boolean;
  required?: boolean;
  format?: (value: any) => string;
}

// Tool behavior configuration
export interface ToolConfig {
  behavior: {
    itemsPerPage: number;
    refreshRate: number;
    defaultStatusFilter: string;
  };
}

// Synchronization statistics
export interface SyncStats {
  shop_id: string;
  total_orders: number;
  synced_orders: number;
  skipped_orders: number;
  error_orders: number;
  last_sync_time: string | null;
  next_scheduled_run: string | null;
  aborted: boolean;
}

// VirtueMart Order interface
export interface VirtueMartOrder {
  virtuemart_order_id: number;
  order_number: string;
  created_on: string;
  order_total: number;
  virtuemart_user_id?: number;
  order_status?: string;
  first_name?: string;
  last_name?: string;
  phone_1?: string;
  phone_2?: string;
  address_1?: string;
  address_2?: string;
  zip?: string;
  city?: string;
  virtuemart_country_id?: number;
  email?: string;
  virtuemart_paymentmethod_id?: number;
  virtuemart_shipmentmethod_id?: number;
  virtuemart_order_userinfo_id?: number;
  customer_note?: string;
  order_shipment?: number;
  coupon_code?: string;
  coupon_discount?: number;
  company?: string;
  shop_id?: string; // Added to track which shop this order belongs to
}

// VirtueMart Order Item interface
export interface VirtueMartOrderItem {
  virtuemart_order_item_id: number;
  virtuemart_order_id: number;
  order_item_sku?: string;
  order_item_name: string;
  product_quantity: number;
  product_final_price: number;
  product_tax?: number;
  product_priceWithoutTax?: number;
}

// Log entry interface
export interface LogEntry {
  timestamp: Date;
  message: string;
  level: string;
  category: string;
  shop_id?: string; // Added to track which shop this log belongs to
}

// JTL Address interface
export interface JtlAddress {
  City: string;
  CountryIso: string;
  Company: string;
  FormOfAddress: string;
  Title: string;
  FirstName: string;
  LastName: string;
  Street: string;
  Address2: string;
  PostalCode: string;
  State: string;
  PhoneNumber: string;
  MobilePhoneNumber: string;
  EmailAddress: string;
  Fax: string;
}

// JTL Customer interface
export interface JtlCustomer {
  CustomerGroupId: number;
  BillingAddress: JtlAddress;
  InternalCompanyId: number;
  LanguageIso: string;
  Shipmentaddress: JtlAddress;
  CustomerSince: string;
  Number: string;
}

// JTL Order interface
export interface JtlOrder {
  CustomerId: number;
  ExternalNumber: string;
  CompanyId: number;
  DepartureCountry: JtlCountry;
  BillingAddress: JtlAddress;
  Shipmentaddress: JtlAddress;
  SalesOrderDate: string;
  SalesOrderPaymentDetails: JtlPaymentDetails;
  SalesOrderShippingDetail: JtlShippingDetails;
  Comment: string;
  LanguageIso: string;
}

// JTL Order Item interface
export interface JtlOrderItem {
  Quantity: number;
  SalesPriceGross: number | null;
  TaxRate: number;
  Name: string;
  SalesUnit: string;
  SalesPriceNet: number | null;
  PurchasePriceNet?: number | null;
}

// JTL Country interface
export interface JtlCountry {
  CountryISO: string;
  CurrencyIso: string;
  CurrencyFactor: number;
}

// JTL Payment Details interface
export interface JtlPaymentDetails {
  PaymentMethodId: number;
  CurrencyIso: string;
  CurrencyFactor: number;
}

// JTL Shipping Details interface
export interface JtlShippingDetails {
  ShippingMethodId: number;
  ShippingDate: string;
}

// Theme interface for styling
export interface Theme {
  name: string;
  label: string;
  colors: Record<string, string>;
}

// Scheduled Job interface
export interface ScheduledJob {
  id: string;
  name: string;
  cronExpression: string;
  lastRun: string | null;
  nextRun: string | null;
  enabled: boolean;
  shop_id: string; // Added to associate jobs with specific shops
}
