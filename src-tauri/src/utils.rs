use std::collections::HashMap;
use log::{info};
use chrono::Utc;
use tauri::Emitter;

use crate::models::{VirtueMartOrder, JtlAddress};

// Standardwert für unbekannte Zahlungsmethoden
const DEFAULT_PAYMENT_METHOD_ID: i32 = 20;

// Mapping von VirtueMart-Zahlungsmethoden zu JTL-Zahlungsmethoden
lazy_static::lazy_static! {
    static ref PAYMENT_METHOD_MAPPING: HashMap<i32, i32> = {
        let mut map = HashMap::new();
        map.insert(2, 38);  // Joomla: Giropay -> JTL: Giropay
        map.insert(14, 4);  // Joomla: Klarna -> JTL: Kreditkarte
        map.insert(4, 2);   // Joomla: Vorauszahlung/Überweisung -> JTL: Überweisung
        map.insert(5, 4);   // Joomla: MasterCard/VISA -> JTL: Kreditkarte
        map.insert(6, 39);  // Joomla: Sofortüberweisung.de -> JTL: Sofortüberweisung
        map.insert(8, 27);  // Joomla: Barzahlung bei Abholung vor Ort -> JTL: Barzahlung
        map.insert(9, 9);   // Joomla: PayPal Express -> JTL: PayPal-Express
        map.insert(10, 34); // Joomla: Amazon Pay -> JTL: Amazon Pay Checkout
        map.insert(17, 10); // Joomla: PayPal Plus -> JTL: PayPal-Plus
        map
    };
}

lazy_static::lazy_static! {
    static ref COUNTRY_MAP: HashMap<i32, &'static str> = {
        let mut map = HashMap::new();
        map.insert(1, "AF");  // Afghanistan
        map.insert(2, "AL");  // Albania
        map.insert(3, "DZ");  // Algeria
        map.insert(4, "AS");  // American Samoa
        map.insert(5, "AD");  // Andorra
        map.insert(6, "AO");  // Angola
        map.insert(7, "AI");  // Anguilla
        map.insert(8, "AQ");  // Antarctica
        map.insert(9, "AG");  // Antigua and Barbuda
        map.insert(10, "AR"); // Argentina
        map.insert(11, "AM"); // Armenia
        map.insert(12, "AW"); // Aruba
        map.insert(13, "AU"); // Australia
        map.insert(14, "AT"); // Austria
        map.insert(15, "AZ"); // Azerbaijan
        map.insert(16, "BS"); // Bahamas
        map.insert(17, "BH"); // Bahrain
        map.insert(18, "BD"); // Bangladesh
        map.insert(19, "BB"); // Barbados
        map.insert(20, "BY"); // Belarus
        map.insert(21, "BE"); // Belgium
        map.insert(22, "BZ"); // Belize
        map.insert(23, "BJ"); // Benin
        map.insert(24, "BM"); // Bermuda
        map.insert(25, "BT"); // Bhutan
        map.insert(26, "BO"); // Bolivia
        map.insert(27, "BA"); // Bosnia and Herzegowina
        map.insert(28, "BW"); // Botswana
        map.insert(29, "BV"); // Bouvet Island
        map.insert(30, "BR"); // Brazil
        map.insert(31, "IO"); // British Indian Ocean Territory
        map.insert(32, "BN"); // Brunei Darussalam
        map.insert(33, "BG"); // Bulgaria
        map.insert(34, "BF"); // Burkina Faso
        map.insert(35, "BI"); // Burundi
        map.insert(36, "KH"); // Cambodia
        map.insert(37, "CM"); // Cameroon
        map.insert(38, "CA"); // Canada
        map.insert(39, "CV"); // Cape Verde
        map.insert(40, "KY"); // Cayman Islands
        map.insert(41, "CF"); // Central African Republic
        map.insert(42, "TD"); // Chad
        map.insert(43, "CL"); // Chile
        map.insert(44, "CN"); // China
        map.insert(45, "CX"); // Christmas Island
        map.insert(46, "CC"); // Cocos (Keeling) Islands
        map.insert(47, "CO"); // Colombia
        map.insert(48, "KM"); // Comoros
        map.insert(49, "CG"); // Congo
        map.insert(50, "CK"); // Cook Islands
        map.insert(51, "CR"); // Costa Rica
        map.insert(52, "CI"); // Cote D'Ivoire
        map.insert(53, "HR"); // Croatia
        map.insert(54, "CU"); // Cuba
        map.insert(55, "CY"); // Cyprus
        map.insert(56, "CZ"); // Czech Republic
        map.insert(57, "DK"); // Denmark
        map.insert(58, "DJ"); // Djibouti
        map.insert(59, "DM"); // Dominica
        map.insert(60, "DO"); // Dominican Republic
        map.insert(61, "TP"); // East Timor
        map.insert(62, "EC"); // Ecuador
        map.insert(63, "EG"); // Egypt
        map.insert(64, "SV"); // El Salvador
        map.insert(65, "GQ"); // Equatorial Guinea
        map.insert(66, "ER"); // Eritrea
        map.insert(67, "EE"); // Estonia
        map.insert(68, "ET"); // Ethiopia
        map.insert(69, "FK"); // Falkland Islands (Malvinas)
        map.insert(70, "FO"); // Faroe Islands
        map.insert(71, "FJ"); // Fiji
        map.insert(72, "FI"); // Finland
        map.insert(73, "FR"); // France
        map.insert(75, "GF"); // French Guiana
        map.insert(76, "PF"); // French Polynesia
        map.insert(77, "TF"); // French Southern Territories
        map.insert(78, "GA"); // Gabon
        map.insert(79, "GM"); // Gambia
        map.insert(80, "GE"); // Georgia
        map.insert(81, "DE"); // Germany
        map.insert(82, "GH"); // Ghana
        map.insert(83, "GI"); // Gibraltar
        map.insert(84, "GR"); // Greece
        map.insert(85, "GL"); // Greenland
        map.insert(86, "GD"); // Grenada
        map.insert(87, "GP"); // Guadeloupe
        map.insert(88, "GU"); // Guam
        map.insert(89, "GT"); // Guatemala
        map.insert(90, "GN"); // Guinea
        map.insert(91, "GW"); // Guinea-bissau
        map.insert(92, "GY"); // Guyana
        map.insert(93, "HT"); // Haiti
        map.insert(94, "HM"); // Heard and Mc Donald Islands
        map.insert(95, "HN"); // Honduras
        map.insert(96, "HK"); // Hong Kong
        map.insert(97, "HU"); // Hungary
        map.insert(98, "IS"); // Iceland
        map.insert(99, "IN"); // India
        map.insert(100, "ID"); // Indonesia
        map.insert(101, "IR"); // Iran (Islamic Republic of)
        map.insert(102, "IQ"); // Iraq
        map.insert(103, "IE"); // Ireland
        map.insert(104, "IL"); // Israel
        map.insert(105, "IT"); // Italy
        map.insert(106, "JM"); // Jamaica
        map.insert(107, "JP"); // Japan
        map.insert(108, "JO"); // Jordan
        map.insert(109, "KZ"); // Kazakhstan
        map.insert(110, "KE"); // Kenya
        map.insert(111, "KI"); // Kiribati
        map.insert(112, "KP"); // Korea, Democratic People's Republic of
        map.insert(113, "KR"); // Korea, Republic of
        map.insert(114, "KW"); // Kuwait
        map.insert(115, "KG"); // Kyrgyzstan
        map.insert(116, "LA"); // Lao People's Democratic Republic
        map.insert(117, "LV"); // Latvia
        map.insert(118, "LB"); // Lebanon
        map.insert(119, "LS"); // Lesotho
        map.insert(120, "LR"); // Liberia
        map.insert(121, "LY"); // Libya
        map.insert(122, "LI"); // Liechtenstein
        map.insert(123, "LT"); // Lithuania
        map.insert(124, "LU"); // Luxembourg
        map.insert(125, "MO"); // Macau
        map.insert(126, "MK"); // Macedonia, The Former Yugoslav Republic of
        map.insert(127, "MG"); // Madagascar
        map.insert(128, "MW"); // Malawi
        map.insert(129, "MY"); // Malaysia
        map.insert(130, "MV"); // Maldives
        map.insert(131, "ML"); // Mali
        map.insert(132, "MT"); // Malta
        map.insert(133, "MH"); // Marshall Islands
        map.insert(134, "MQ"); // Martinique
        map.insert(135, "MR"); // Mauritania
        map.insert(136, "MU"); // Mauritius
        map.insert(137, "YT"); // Mayotte
        map.insert(138, "MX"); // Mexico
        map.insert(139, "FM"); // Micronesia, Federated States of
        map.insert(140, "MD"); // Moldova, Republic of
        map.insert(141, "MC"); // Monaco
        map.insert(142, "MN"); // Mongolia
        map.insert(143, "MS"); // Montserrat
        map.insert(144, "MA"); // Morocco
        map.insert(145, "MZ"); // Mozambique
        map.insert(146, "MM"); // Myanmar
        map.insert(147, "NA"); // Namibia
        map.insert(148, "NR"); // Nauru
        map.insert(149, "NP"); // Nepal
        map.insert(150, "NL"); // Netherlands
        map.insert(151, "AN"); // Netherlands Antilles
        map.insert(152, "NC"); // New Caledonia
        map.insert(153, "NZ"); // New Zealand
        map.insert(154, "NI"); // Nicaragua
        map.insert(155, "NE"); // Niger
        map.insert(156, "NG"); // Nigeria
        map.insert(157, "NU"); // Niue
        map.insert(158, "NF"); // Norfolk Island
        map.insert(159, "MP"); // Northern Mariana Islands
        map.insert(160, "NO"); // Norway
        map.insert(161, "OM"); // Oman
        map.insert(162, "PK"); // Pakistan
        map.insert(163, "PW"); // Palau
        map.insert(164, "PA"); // Panama
        map.insert(165, "PG"); // Papua New Guinea
        map.insert(166, "PY"); // Paraguay
        map.insert(167, "PE"); // Peru
        map.insert(168, "PH"); // Philippines
        map.insert(169, "PN"); // Pitcairn
        map.insert(170, "PL"); // Poland
        map.insert(171, "PT"); // Portugal
        map.insert(172, "PR"); // Puerto Rico
        map.insert(173, "QA"); // Qatar
        map.insert(174, "RE"); // Reunion
        map.insert(175, "RO"); // Romania
        map.insert(176, "RU"); // Russian Federation
        map.insert(177, "RW"); // Rwanda
        map.insert(178, "KN"); // Saint Kitts and Nevis
        map.insert(179, "LC"); // Saint Lucia
        map.insert(180, "VC"); // Saint Vincent and the Grenadines
        map.insert(181, "WS"); // Samoa
        map.insert(182, "SM"); // San Marino
        map.insert(183, "ST"); // Sao Tome and Principe
        map.insert(184, "SA"); // Saudi Arabia
        map.insert(185, "SN"); // Senegal
        map.insert(186, "SC"); // Seychelles
        map.insert(187, "SL"); // Sierra Leone
        map.insert(188, "SG"); // Singapore
        map.insert(189, "SK"); // Slovakia
        map.insert(190, "SI"); // Slovenia
        map.insert(191, "SB"); // Solomon Islands
        map.insert(192, "SO"); // Somalia
        map.insert(193, "ZA"); // South Africa
        map.insert(194, "GS"); // South Georgia and the South Sandwich Islands
        map.insert(195, "ES"); // Spain
        map.insert(196, "LK"); // Sri Lanka
        map.insert(197, "SH"); // St. Helena
        map.insert(198, "PM"); // St. Pierre and Miquelon
        map.insert(199, "SD"); // Sudan
        map.insert(200, "SR"); // Suriname
        map.insert(201, "SJ"); // Svalbard and Jan Mayen Islands
        map.insert(202, "SZ"); // Swaziland
        map.insert(203, "SE"); // Sweden
        map.insert(204, "CH"); // Switzerland
        map.insert(205, "SY"); // Syrian Arab Republic
        map.insert(206, "TW"); // Taiwan
        map.insert(207, "TJ"); // Tajikistan
        map.insert(208, "TZ"); // Tanzania, United Republic of
        map.insert(209, "TH"); // Thailand
        map.insert(210, "TG"); // Togo
        map.insert(211, "TK"); // Tokelau
        map.insert(212, "TO"); // Tonga
        map.insert(213, "TT"); // Trinidad and Tobago
        map.insert(214, "TN"); // Tunisia
        map.insert(215, "TR"); // Turkey
        map.insert(216, "TM"); // Turkmenistan
        map.insert(217, "TC"); // Turks and Caicos Islands
        map.insert(218, "TV"); // Tuvalu
        map.insert(219, "UG"); // Uganda
        map.insert(220, "UA"); // Ukraine
        map.insert(221, "AE"); // United Arab Emirates
        map.insert(222, "GB"); // United Kingdom
        map.insert(223, "US"); // United States
        map.insert(224, "UM"); // United States Minor Outlying Islands
        map.insert(225, "UY"); // Uruguay
        map.insert(226, "UZ"); // Uzbekistan
        map.insert(227, "VU"); // Vanuatu
        map.insert(228, "VA"); // Vatican City State (Holy See)
        map.insert(229, "VE"); // Venezuela
        map.insert(230, "VN"); // Viet Nam
        map.insert(231, "VG"); // Virgin Islands (British)
        map.insert(232, "VI"); // Virgin Islands (U.S.)
        map.insert(233, "WF"); // Wallis and Futuna Islands
        map.insert(234, "EH"); // Western Sahara
        map.insert(235, "YE"); // Yemen
        map.insert(237, "DC"); // The Democratic Republic of Congo
        map.insert(238, "ZM"); // Zambia
        map.insert(239, "ZW"); // Zimbabwe
        map.insert(240, "XE"); // East Timor
        map.insert(241, "JE"); // Jersey
        map.insert(242, "XB"); // St. Barthelemy
        map.insert(243, "XU"); // St. Eustatius
        map.insert(244, "XC"); // Canary Islands
        map.insert(245, "RS"); // Serbia
        map.insert(246, "MF"); // Sint Maarten (French Antilles)
        map.insert(247, "SX"); // Sint Maarten (Netherlands Antilles)
        map.insert(248, "PS"); // Palestinian Territory, occupied
        map
    };
}

pub fn get_country_code(id: i32) -> Option<&'static str> {
    COUNTRY_MAP.get(&id).copied()
}

pub fn emit_event<R: tauri::Runtime, T: serde::Serialize + Clone>(
    app_handle: &tauri::AppHandle<R>,
    event: &str, 
    payload: T
) -> Result<(), String> {
    app_handle
        .emit(event, payload)
        .map_err(|e| format!("Failed to emit event: {}", e))
}


/// Mappt die VirtueMart-Zahlungsmethode auf die JTL-Zahlungsmethode
pub fn map_payment_method(payment_method_id: Option<i32>) -> i32 {
    match payment_method_id {
        Some(id) => {
            match PAYMENT_METHOD_MAPPING.get(&id) {
                Some(&jtl_id) => jtl_id,
                None => {
                    info!("Unbekannte Zahlungsmethoden-ID: {}, verwende Standard: {}", 
                          id, DEFAULT_PAYMENT_METHOD_ID);
                    DEFAULT_PAYMENT_METHOD_ID
                }
            }
        },
        None => {
            info!("Keine Zahlungsmethoden-ID angegeben, verwende Standard: {}", 
                  DEFAULT_PAYMENT_METHOD_ID);
            DEFAULT_PAYMENT_METHOD_ID
        }
    }
}

/// Erzeugt ein JTL-Adressobjekt aus einem VirtueMart-Adressobjekt
pub fn create_address_object(address_data: &VirtueMartOrder) -> JtlAddress {
    JtlAddress {
        City: address_data.city.clone().unwrap_or_default(),
        CountryIso: get_country_code(address_data.virtuemart_country_id.clone().unwrap_or_default()).unwrap_or_default().to_string(),
        Company: address_data.company.clone().unwrap_or_default(), // VirtueMart-Modell hat kein Unternehmen in deinem Beispiel
        FormOfAddress: String::new(), // VirtueMart-Modell hat keine Anrede in deinem Beispiel
        Title: String::new(),
        FirstName: address_data.first_name.clone().unwrap_or_default(),
        LastName: address_data.last_name.clone().unwrap_or_default(),
        Street: format!("{}{}", 
            address_data.address_1.clone().unwrap_or_default(),
            address_data.address_2.clone().map_or("".to_string(), |a| format!(" {}", a))
        ),
        Address2: String::new(),
        PostalCode: address_data.zip.clone().unwrap_or_default(),
        State: String::new(),
        PhoneNumber: address_data.phone_1.clone().unwrap_or_default(), // Füge Phone-Feld hinzu, wenn verfügbar
        MobilePhoneNumber: String::new(),
        EmailAddress: address_data.email.clone().unwrap_or_default(),
        Fax: String::new(),
    }
}

/// Erzeugt eine Timestamp für Logs
pub fn get_timestamp() -> String {
    Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Hilfsfunktion zum Parsen von Float-Werten aus Strings
pub fn parse_float(value: Option<&str>) -> f64 {
    match value {
        Some(val) => val.parse::<f64>().unwrap_or(0.0),
        None => 0.0,
    }
}

pub fn format_iso_date(date_str: &str) -> String {
    // Try to parse the input date format
    if let Ok(dt) = chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S") {
        // Convert to DateTime<Utc> and format as ISO 8601
        return chrono::DateTime::<Utc>::from_utc(dt, Utc).to_rfc3339();
    }
    
    // If parsing fails, return a default date format
    Utc::now().to_rfc3339()
}