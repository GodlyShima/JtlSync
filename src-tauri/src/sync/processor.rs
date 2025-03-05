use log::{info, warn};
use mysql::Pool;

use crate::api::jtl::JtlApiClient;
use crate::config::shop::ShopConfig;
use crate::db::joomla::{get_order_items, get_shipping_address};
use crate::db::models::{VirtueMartOrder, JtlOrder, JtlAddress, JtlOrderItem, JtlCustomer, JtlCountry, JtlPaymentDetails, JtlShippingDetails};
use crate::error::{Result, Error};
use crate::utils::mapping::{map_payment_method, create_address_object, get_country_code};
use crate::utils::format::format_iso_date;

/// Process a single order for synchronization
/// 
/// Returns Ok(true) if order was successfully synced
/// Returns Ok(false) if order was skipped (already exists)
/// Returns Err if there was an error during processing
pub async fn process_order(
    client: &JtlApiClient,
    joomla_conn: &Pool,
    order: &VirtueMartOrder,
    shop: &ShopConfig
) -> Result<bool> {
    // Create customer number with shop ID prefix for uniqueness between shops
    let customer_number = format!("VM{}", order.virtuemart_order_userinfo_id.unwrap_or_default().to_string());
    
    info!("Customer number from Joomla for shop '{}': {}", shop.name, customer_number);
    
    // Get shipping address
    let shipping_address = get_shipping_address(joomla_conn, shop, order.virtuemart_order_id)?;
    
    // Map payment method
    let jtl_payment_method_id = map_payment_method(order.virtuemart_paymentmethod_id);
    
    // Order number with shop ID prefix for uniqueness between shops
    let order_number = format!("VM{}", order.virtuemart_order_id);

    
    // Check if customer already exists
    let customer_id = match client.get_customer_by_id(&customer_number).await? {
        Some(customer) => {
            info!("Customer {} already exists with ID: {} (Shop: '{}')", 
                  customer_number, customer["Id"], shop.name);
            customer["Id"].as_str().unwrap_or("0").to_string()
        },
        None => {
            // Create new customer
            info!("Creating new customer {} for shop '{}'", customer_number, shop.name);
            
            let billing_address = create_address_object(order);
            let shipping_addr = match &shipping_address {
                Some(addr) => create_address_object(addr),
                None => billing_address.clone(),
            };
            
            let customer_data = JtlCustomer {
                CustomerGroupId: 1,
                BillingAddress: billing_address,
                InternalCompanyId: 1,
                LanguageIso: "DE".to_string(),
                Shipmentaddress: shipping_addr,
                CustomerSince: format_iso_date(&order.created_on),
                Number: customer_number.clone(),
            };
            
            let response = client.create_customer(&customer_data).await?;
            info!("Customer created with ID: {} for shop '{}'", response["Id"], shop.name);
            response["Id"].to_string()
        }
    };
    
    // Check if order already exists
    if client.check_order_exists(&order_number, &customer_id).await? {
        warn!("Order {} already exists for shop '{}', skipping", 
              order_number, shop.name);
        return Ok(false);
    }
    
    // Get order items
    let items = get_order_items(joomla_conn, shop, order.virtuemart_order_id)?;
    
    info!("Found {} order items for shop '{}'", items.len(), shop.name);
    
    // Create JTL order
    info!("Creating order {} in JTL for shop '{}'", order_number, shop.name);
    
    let billing_address = create_address_object(order);
    let shipping_addr = match &shipping_address {
        Some(addr) => create_address_object(addr),
        None => billing_address.clone(),
    };

    info!("CustomerId: {} for shop '{}'", customer_id.clone(), shop.name);
    info!("ExternalNumber: {} for shop '{}'", order_number.clone(), shop.name);
    info!("Country: {} ID: {} for shop '{}'", 
          get_country_code(order.virtuemart_country_id.unwrap_or_default()).unwrap_or_default(), 
          order.virtuemart_country_id.unwrap_or_default(),
          shop.name);

    let jtl_order = JtlOrder {
        CustomerId: customer_id.parse::<i32>().unwrap_or_default(),
        ExternalNumber: order_number.clone(),
        CompanyId: 1,
        DepartureCountry: JtlCountry {
            CountryISO: "DE".to_string(),
            CurrencyIso: "EUR".to_string(),
            CurrencyFactor: 1.0,
        },
        BillingAddress: billing_address,
        Shipmentaddress: shipping_addr,
        SalesOrderDate: format_iso_date(&order.created_on),
        SalesOrderPaymentDetails: JtlPaymentDetails {
            PaymentMethodId: jtl_payment_method_id,
            CurrencyIso: "EUR".to_string(),
            CurrencyFactor: 1.0,
        },
        SalesOrderShippingDetail: JtlShippingDetails {
            ShippingMethodId: 7, // Standard shipping method
            ShippingDate: format_iso_date(&order.created_on),
        },
        Comment: format!("Shop: {} - {}", shop.name, order.customer_note.clone().unwrap_or_default()),
        LanguageIso: "DE".to_string(),
    };
    
    // Prepare order items for JTL
    let mut all_items: Vec<JtlOrderItem> = items.iter().map(|item| {
        JtlOrderItem {
            Quantity: item.product_quantity,
            SalesPriceGross: Some(item.product_final_price),
            TaxRate: 19.0,
            Name: format!("[{}] {}", shop.name, item.order_item_name.clone()),
            SalesUnit: "stk".to_string(),
            SalesPriceNet: Some(item.product_priceWithoutTax.unwrap_or(item.product_final_price / 1.19)),
            PurchasePriceNet: None,
        }
    }).collect();

    // Add coupon if present
    if let Some(coupon_code) = &order.coupon_code {
        if !coupon_code.is_empty() {
            let discount = order.coupon_discount.unwrap_or_default();
            all_items.push(JtlOrderItem {
                Quantity: 1,
                SalesPriceGross: Some(discount),
                TaxRate: 0.0,
                Name: format!("[{}] Coupon: {}", shop.name, coupon_code),
                SalesUnit: "stk".to_string(),
                PurchasePriceNet: None,
                SalesPriceNet: Some(discount),
            });
        }
    }

    // Add shipping if present
    if let Some(shipping_cost) = order.order_shipment {
        if shipping_cost > 0.0 {
            all_items.push(JtlOrderItem {
                Quantity: 1,
                SalesPriceGross: Some(shipping_cost),
                TaxRate: 19.0,
                Name: format!("[{}] Shipping", shop.name),
                SalesUnit: "stk".to_string(),
                SalesPriceNet: Some(shipping_cost / 1.19),
                PurchasePriceNet: None,
            });
        }
    }
    
    // Create order in JTL
    let response = client.create_order(&jtl_order, &all_items).await?;
    let order_id = response["Id"].to_string();
    info!("Order {} successfully created in JTL with ID: {} for shop '{}'", 
          order_number, order_id, shop.name);
    
    // If already paid
    if let Some(status) = &order.order_status {
        if status == "C" && jtl_payment_method_id != 4 {
            info!("Order {} is paid -> setting to paid for shop '{}'", 
                  order_number, shop.name);
            let _ = client.set_payment_paid(&order_id).await;
        }
    }

    // Set order on hold
    let _ = client.set_order_hold(&order_id).await;
    
    Ok(true)
}