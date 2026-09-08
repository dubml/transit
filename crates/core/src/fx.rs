//! Published FX: U.S. Treasury Reporting Rates of Exchange.
//! Units of foreign currency per 1 USD. Integer micro-units (1e-6).
//! Unknown country/currency fails closed. No invented rates.

use serde::Serialize;

pub const FX_SOURCE: &str =
    "https://fiscaldata.treasury.gov/datasets/treasury-reporting-rates-exchange/";
pub const FX_AS_OF: &str = "2026-06-30";
pub const FX_RATE_SCALE: u128 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FxRate {
    pub country: &'static str,
    pub currency: &'static str,
    pub currency_code: &'static str,
    pub description: &'static str,
    /// Foreign currency micro-units per 1 USD.
    pub units_per_usd_micros: u128,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FxError {
    UnknownMarket { query: String },
}

impl std::fmt::Display for FxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownMarket { query } => {
                write!(f, "no published Treasury FX row for {query}")
            }
        }
    }
}
impl std::error::Error for FxError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FxQuote {
    pub country: &'static str,
    pub currency: &'static str,
    pub currency_code: &'static str,
    pub units_per_usd: String,
    pub amount: String,
    pub source: &'static str,
    pub as_of: &'static str,
}

const RATES: &[FxRate] = &[
    FxRate {
        country: "United States",
        currency: "Dollar",
        currency_code: "USD",
        description: "United States-Dollar",
        units_per_usd_micros: 1000000,
    },
    FxRate {
        country: "Austria",
        currency: "Euro",
        currency_code: "EUR",
        description: "Austria-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Belgium",
        currency: "Euro",
        currency_code: "EUR",
        description: "Belgium-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Croatia",
        currency: "Euro",
        currency_code: "EUR",
        description: "Croatia-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Estonia",
        currency: "Euro",
        currency_code: "EUR",
        description: "Estonia-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Finland",
        currency: "Euro",
        currency_code: "EUR",
        description: "Finland-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "France",
        currency: "Euro",
        currency_code: "EUR",
        description: "France-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Germany",
        currency: "Euro",
        currency_code: "EUR",
        description: "Germany-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Greece",
        currency: "Euro",
        currency_code: "EUR",
        description: "Greece-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Ireland",
        currency: "Euro",
        currency_code: "EUR",
        description: "Ireland-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Italy",
        currency: "Euro",
        currency_code: "EUR",
        description: "Italy-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Latvia",
        currency: "Euro",
        currency_code: "EUR",
        description: "Latvia-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Lithuania",
        currency: "Euro",
        currency_code: "EUR",
        description: "Lithuania-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Luxembourg",
        currency: "Euro",
        currency_code: "EUR",
        description: "Luxembourg-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Malta",
        currency: "Euro",
        currency_code: "EUR",
        description: "Malta-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Netherlands",
        currency: "Euro",
        currency_code: "EUR",
        description: "Netherlands-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Portugal",
        currency: "Euro",
        currency_code: "EUR",
        description: "Portugal-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Slovakia",
        currency: "Euro",
        currency_code: "EUR",
        description: "Slovakia-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Slovenia",
        currency: "Euro",
        currency_code: "EUR",
        description: "Slovenia-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Spain",
        currency: "Euro",
        currency_code: "EUR",
        description: "Spain-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Afghanistan",
        currency: "Afghani",
        currency_code: "AFN",
        description: "Afghanistan-Afghani",
        units_per_usd_micros: 65090000,
    },
    FxRate {
        country: "Albania",
        currency: "Lek",
        currency_code: "ALL",
        description: "Albania-Lek",
        units_per_usd_micros: 82400000,
    },
    FxRate {
        country: "Algeria",
        currency: "Dinar",
        currency_code: "DZD",
        description: "Algeria-Dinar",
        units_per_usd_micros: 132906000,
    },
    FxRate {
        country: "Angola",
        currency: "Kwanza",
        currency_code: "AOA",
        description: "Angola-Kwanza",
        units_per_usd_micros: 912048000,
    },
    FxRate {
        country: "Antigua & Barbuda",
        currency: "East Caribbean Dollar",
        currency_code: "XCD",
        description: "Antigua & Barbuda-East Caribbean Dollar",
        units_per_usd_micros: 2700000,
    },
    FxRate {
        country: "Argentina",
        currency: "Peso",
        currency_code: "ARS",
        description: "Argentina-Peso",
        units_per_usd_micros: 1495000000,
    },
    FxRate {
        country: "Armenia",
        currency: "Dram",
        currency_code: "AMD",
        description: "Armenia-Dram",
        units_per_usd_micros: 370000000,
    },
    FxRate {
        country: "Australia",
        currency: "Dollar",
        currency_code: "AUD",
        description: "Australia-Dollar",
        units_per_usd_micros: 1451000,
    },
    FxRate {
        country: "Azerbaijan",
        currency: "Manat",
        currency_code: "AZN",
        description: "Azerbaijan-Manat",
        units_per_usd_micros: 1700000,
    },
    FxRate {
        country: "Bahamas",
        currency: "Dollar",
        currency_code: "BSD",
        description: "Bahamas-Dollar",
        units_per_usd_micros: 1000000,
    },
    FxRate {
        country: "Bahrain",
        currency: "Dinar",
        currency_code: "BHD",
        description: "Bahrain-Dinar",
        units_per_usd_micros: 377000,
    },
    FxRate {
        country: "Bangladesh",
        currency: "Taka",
        currency_code: "BDT",
        description: "Bangladesh-Taka",
        units_per_usd_micros: 123000000,
    },
    FxRate {
        country: "Barbados",
        currency: "Dollar",
        currency_code: "BBD",
        description: "Barbados-Dollar",
        units_per_usd_micros: 2020000,
    },
    FxRate {
        country: "Belarus",
        currency: "New Ruble",
        currency_code: "BYN",
        description: "Belarus-New Ruble",
        units_per_usd_micros: 2907000,
    },
    FxRate {
        country: "Belize",
        currency: "Dollar",
        currency_code: "BZD",
        description: "Belize-Dollar",
        units_per_usd_micros: 2000000,
    },
    FxRate {
        country: "Benin",
        currency: "Cfa Franc",
        currency_code: "XOF",
        description: "Benin-Cfa Franc",
        units_per_usd_micros: 571500000,
    },
    FxRate {
        country: "Bermuda",
        currency: "Dollar",
        currency_code: "BMD",
        description: "Bermuda-Dollar",
        units_per_usd_micros: 1000000,
    },
    FxRate {
        country: "Bolivia",
        currency: "Boliviano",
        currency_code: "BOB",
        description: "Bolivia-Boliviano",
        units_per_usd_micros: 6850000,
    },
    FxRate {
        country: "Bosnia",
        currency: "Marka",
        currency_code: "BAM",
        description: "Bosnia-Marka",
        units_per_usd_micros: 1716000,
    },
    FxRate {
        country: "Botswana",
        currency: "Pula",
        currency_code: "BWP",
        description: "Botswana-Pula",
        units_per_usd_micros: 12821000,
    },
    FxRate {
        country: "Brazil",
        currency: "Real",
        currency_code: "BRL",
        description: "Brazil-Real",
        units_per_usd_micros: 5174000,
    },
    FxRate {
        country: "Brunei",
        currency: "Dollar",
        currency_code: "BND",
        description: "Brunei-Dollar",
        units_per_usd_micros: 1295000,
    },
    FxRate {
        country: "Bulgaria",
        currency: "Lev New",
        currency_code: "BGN",
        description: "Bulgaria-Lev New",
        units_per_usd_micros: 870000,
    },
    FxRate {
        country: "Burkina Faso",
        currency: "Cfa Franc",
        currency_code: "XOF",
        description: "Burkina Faso-Cfa Franc",
        units_per_usd_micros: 571500000,
    },
    FxRate {
        country: "Burundi",
        currency: "Franc",
        currency_code: "BIF",
        description: "Burundi-Franc",
        units_per_usd_micros: 3000000000,
    },
    FxRate {
        country: "Cambodia",
        currency: "Riel",
        currency_code: "KHR",
        description: "Cambodia-Riel",
        units_per_usd_micros: 4020500000,
    },
    FxRate {
        country: "Cameroon",
        currency: "Cfa Franc",
        currency_code: "XAF",
        description: "Cameroon-Cfa Franc",
        units_per_usd_micros: 575550000,
    },
    FxRate {
        country: "Canada",
        currency: "Dollar",
        currency_code: "CAD",
        description: "Canada-Dollar",
        units_per_usd_micros: 1424000,
    },
    FxRate {
        country: "Cape Verde",
        currency: "Escudo",
        currency_code: "CVE",
        description: "Cape Verde-Escudo",
        units_per_usd_micros: 96770000,
    },
    FxRate {
        country: "Cayman Islands",
        currency: "Dollar",
        currency_code: "KYD",
        description: "Cayman Islands-Dollar",
        units_per_usd_micros: 820000,
    },
    FxRate {
        country: "Central African Republic",
        currency: "Cfa Franc",
        currency_code: "XAF",
        description: "Central African Republic-Cfa Franc",
        units_per_usd_micros: 575550000,
    },
    FxRate {
        country: "Chad",
        currency: "Cfa Franc",
        currency_code: "XAF",
        description: "Chad-Cfa Franc",
        units_per_usd_micros: 575550000,
    },
    FxRate {
        country: "Chile",
        currency: "Peso",
        currency_code: "CLP",
        description: "Chile-Peso",
        units_per_usd_micros: 921600000,
    },
    FxRate {
        country: "China",
        currency: "Renminbi",
        currency_code: "CNY",
        description: "China-Renminbi",
        units_per_usd_micros: 6785000,
    },
    FxRate {
        country: "Colombia",
        currency: "Peso",
        currency_code: "COP",
        description: "Colombia-Peso",
        units_per_usd_micros: 3449190000,
    },
    FxRate {
        country: "Comoros",
        currency: "Franc",
        currency_code: "KMF",
        description: "Comoros-Franc",
        units_per_usd_micros: 431300000,
    },
    FxRate {
        country: "Congo",
        currency: "Cfa Franc",
        currency_code: "XAF",
        description: "Congo-Cfa Franc",
        units_per_usd_micros: 575550000,
    },
    FxRate {
        country: "Costa Rica",
        currency: "Colon",
        currency_code: "CRC",
        description: "Costa Rica-Colon",
        units_per_usd_micros: 451950000,
    },
    FxRate {
        country: "Cote D'Ivoire",
        currency: "Cfa Franc",
        currency_code: "XOF",
        description: "Cote D'Ivoire-Cfa Franc",
        units_per_usd_micros: 571500000,
    },
    FxRate {
        country: "Cuba",
        currency: "Chavito",
        currency_code: "CUP",
        description: "Cuba-Chavito",
        units_per_usd_micros: 1000000,
    },
    FxRate {
        country: "Cuba",
        currency: "Peso",
        currency_code: "CUP",
        description: "Cuba-Peso",
        units_per_usd_micros: 24000000,
    },
    FxRate {
        country: "Curacao",
        currency: "Caribbean Guilder",
        currency_code: "XCG",
        description: "Curacao-Caribbean Guilder",
        units_per_usd_micros: 1780000,
    },
    FxRate {
        country: "Cyprus",
        currency: "Euro",
        currency_code: "EUR",
        description: "Cyprus-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Czech Republic",
        currency: "Koruna",
        currency_code: "CZK",
        description: "Czech Republic-Koruna",
        units_per_usd_micros: 20716000,
    },
    FxRate {
        country: "Democratic Republic Of Congo",
        currency: "Congolese Franc",
        currency_code: "CDF",
        description: "Democratic Republic Of Congo-Congolese Franc",
        units_per_usd_micros: 2277000000,
    },
    FxRate {
        country: "Denmark",
        currency: "Krone",
        currency_code: "DKK",
        description: "Denmark-Krone",
        units_per_usd_micros: 6558000,
    },
    FxRate {
        country: "Djibouti",
        currency: "Franc",
        currency_code: "DJF",
        description: "Djibouti-Franc",
        units_per_usd_micros: 177000000,
    },
    FxRate {
        country: "Dominican Republic",
        currency: "Peso",
        currency_code: "DOP",
        description: "Dominican Republic-Peso",
        units_per_usd_micros: 59360000,
    },
    FxRate {
        country: "Ecuador",
        currency: "Dolares",
        currency_code: "USD",
        description: "Ecuador-Dolares",
        units_per_usd_micros: 1000000,
    },
    FxRate {
        country: "Egypt",
        currency: "Pound",
        currency_code: "EGP",
        description: "Egypt-Pound",
        units_per_usd_micros: 49180000,
    },
    FxRate {
        country: "El Salvador",
        currency: "Dollar",
        currency_code: "USD",
        description: "El Salvador-Dollar",
        units_per_usd_micros: 1000000,
    },
    FxRate {
        country: "Equatorial Guinea",
        currency: "Cfa Franc",
        currency_code: "XAF",
        description: "Equatorial Guinea-Cfa Franc",
        units_per_usd_micros: 575550000,
    },
    FxRate {
        country: "Eritrea",
        currency: "Nakfa",
        currency_code: "ERN",
        description: "Eritrea-Nakfa",
        units_per_usd_micros: 15000000,
    },
    FxRate {
        country: "Eswatini",
        currency: "Lilangeni",
        currency_code: "SZL",
        description: "Eswatini-Lilangeni",
        units_per_usd_micros: 16368000,
    },
    FxRate {
        country: "Ethiopia",
        currency: "Birr",
        currency_code: "ETB",
        description: "Ethiopia-Birr",
        units_per_usd_micros: 158480000,
    },
    FxRate {
        country: "Euro Zone",
        currency: "Euro",
        currency_code: "EUR",
        description: "Euro Zone-Euro",
        units_per_usd_micros: 877000,
    },
    FxRate {
        country: "Fiji",
        currency: "Dollar",
        currency_code: "FJD",
        description: "Fiji-Dollar",
        units_per_usd_micros: 2220000,
    },
    FxRate {
        country: "Gabon",
        currency: "Cfa Franc",
        currency_code: "XAF",
        description: "Gabon-Cfa Franc",
        units_per_usd_micros: 575550000,
    },
    FxRate {
        country: "Gambia",
        currency: "Dalasi",
        currency_code: "GMD",
        description: "Gambia-Dalasi",
        units_per_usd_micros: 72000000,
    },
    FxRate {
        country: "Georgia",
        currency: "Lari",
        currency_code: "GEL",
        description: "Georgia-Lari",
        units_per_usd_micros: 2607000,
    },
    FxRate {
        country: "Ghana",
        currency: "Cedi",
        currency_code: "GHS",
        description: "Ghana-Cedi",
        units_per_usd_micros: 11300000,
    },
    FxRate {
        country: "Grenada",
        currency: "East Caribbean Dollar",
        currency_code: "XCD",
        description: "Grenada-East Caribbean Dollar",
        units_per_usd_micros: 2700000,
    },
    FxRate {
        country: "Guatemala",
        currency: "Quetzal",
        currency_code: "GTQ",
        description: "Guatemala-Quetzal",
        units_per_usd_micros: 7620000,
    },
    FxRate {
        country: "Guinea",
        currency: "Franc",
        currency_code: "GNF",
        description: "Guinea-Franc",
        units_per_usd_micros: 8733000000,
    },
    FxRate {
        country: "Guinea Bissau",
        currency: "Cfa Franc",
        currency_code: "XOF",
        description: "Guinea Bissau-Cfa Franc",
        units_per_usd_micros: 571500000,
    },
    FxRate {
        country: "Guyana",
        currency: "Dollar",
        currency_code: "GYD",
        description: "Guyana-Dollar",
        units_per_usd_micros: 215000000,
    },
    FxRate {
        country: "Haiti",
        currency: "Gourde",
        currency_code: "HTG",
        description: "Haiti-Gourde",
        units_per_usd_micros: 130252000,
    },
    FxRate {
        country: "Honduras",
        currency: "Lempira",
        currency_code: "HNL",
        description: "Honduras-Lempira",
        units_per_usd_micros: 26668000,
    },
    FxRate {
        country: "Hong Kong",
        currency: "Dollar",
        currency_code: "HKD",
        description: "Hong Kong-Dollar",
        units_per_usd_micros: 7842000,
    },
    FxRate {
        country: "Hungary",
        currency: "Forint",
        currency_code: "HUF",
        description: "Hungary-Forint",
        units_per_usd_micros: 312340000,
    },
    FxRate {
        country: "Iceland",
        currency: "Krona",
        currency_code: "ISK",
        description: "Iceland-Krona",
        units_per_usd_micros: 126220000,
    },
    FxRate {
        country: "India",
        currency: "Rupee",
        currency_code: "INR",
        description: "India-Rupee",
        units_per_usd_micros: 94660000,
    },
    FxRate {
        country: "Indonesia",
        currency: "Rupiah",
        currency_code: "IDR",
        description: "Indonesia-Rupiah",
        units_per_usd_micros: 17870690000,
    },
    FxRate {
        country: "Iran",
        currency: "Rial",
        currency_code: "IRR",
        description: "Iran-Rial",
        units_per_usd_micros: 1375055000000,
    },
    FxRate {
        country: "Iraq",
        currency: "Dinar",
        currency_code: "IQD",
        description: "Iraq-Dinar",
        units_per_usd_micros: 1309550000,
    },
    FxRate {
        country: "Israel",
        currency: "Shekel",
        currency_code: "ILS",
        description: "Israel-Shekel",
        units_per_usd_micros: 2977000,
    },
    FxRate {
        country: "Jamaica",
        currency: "Dollar",
        currency_code: "JMD",
        description: "Jamaica-Dollar",
        units_per_usd_micros: 159000000,
    },
    FxRate {
        country: "Japan",
        currency: "Yen",
        currency_code: "JPY",
        description: "Japan-Yen",
        units_per_usd_micros: 162380000,
    },
    FxRate {
        country: "Jordan",
        currency: "Dinar",
        currency_code: "JOD",
        description: "Jordan-Dinar",
        units_per_usd_micros: 708000,
    },
    FxRate {
        country: "Kazakhstan",
        currency: "Tenge",
        currency_code: "KZT",
        description: "Kazakhstan-Tenge",
        units_per_usd_micros: 478110000,
    },
    FxRate {
        country: "Kenya",
        currency: "Shilling",
        currency_code: "KES",
        description: "Kenya-Shilling",
        units_per_usd_micros: 129350000,
    },
    FxRate {
        country: "Korea",
        currency: "Won",
        currency_code: "KRW",
        description: "Korea-Won",
        units_per_usd_micros: 1550880000,
    },
    FxRate {
        country: "Kuwait",
        currency: "Dinar",
        currency_code: "KWD",
        description: "Kuwait-Dinar",
        units_per_usd_micros: 308000,
    },
    FxRate {
        country: "Kyrgyzstan",
        currency: "Som",
        currency_code: "KGS",
        description: "Kyrgyzstan-Som",
        units_per_usd_micros: 87450000,
    },
    FxRate {
        country: "Laos",
        currency: "Kip",
        currency_code: "LAK",
        description: "Laos-Kip",
        units_per_usd_micros: 22320000000,
    },
    FxRate {
        country: "Lebanon",
        currency: "Pound",
        currency_code: "LBP",
        description: "Lebanon-Pound",
        units_per_usd_micros: 89500000000,
    },
    FxRate {
        country: "Lesotho",
        currency: "Maloti",
        currency_code: "LSL",
        description: "Lesotho-Maloti",
        units_per_usd_micros: 16368000,
    },
    FxRate {
        country: "Liberia",
        currency: "Dollar",
        currency_code: "LRD",
        description: "Liberia-Dollar",
        units_per_usd_micros: 181000000,
    },
    FxRate {
        country: "Libya",
        currency: "Dinar",
        currency_code: "LYD",
        description: "Libya-Dinar",
        units_per_usd_micros: 6413000,
    },
    FxRate {
        country: "Madagascar",
        currency: "Ariary",
        currency_code: "MGA",
        description: "Madagascar-Ariary",
        units_per_usd_micros: 4180000000,
    },
    FxRate {
        country: "Malawi",
        currency: "Kwacha",
        currency_code: "MWK",
        description: "Malawi-Kwacha",
        units_per_usd_micros: 1751000000,
    },
    FxRate {
        country: "Malaysia",
        currency: "Ringgit",
        currency_code: "MYR",
        description: "Malaysia-Ringgit",
        units_per_usd_micros: 4082000,
    },
    FxRate {
        country: "Maldives",
        currency: "Rufiyaa",
        currency_code: "MVR",
        description: "Maldives-Rufiyaa",
        units_per_usd_micros: 15420000,
    },
    FxRate {
        country: "Mali",
        currency: "Cfa Franc",
        currency_code: "XOF",
        description: "Mali-Cfa Franc",
        units_per_usd_micros: 571500000,
    },
    FxRate {
        country: "Marshall Islands",
        currency: "U.S. Dollar",
        currency_code: "USD",
        description: "Marshall Islands-U.S. Dollar",
        units_per_usd_micros: 1000000,
    },
    FxRate {
        country: "Mauritania",
        currency: "Ouguiya",
        currency_code: "MRU",
        description: "Mauritania-Ouguiya",
        units_per_usd_micros: 39900000,
    },
    FxRate {
        country: "Mauritius",
        currency: "Rupee",
        currency_code: "MUR",
        description: "Mauritius-Rupee",
        units_per_usd_micros: 47040000,
    },
    FxRate {
        country: "Mexico",
        currency: "Peso",
        currency_code: "MXN",
        description: "Mexico-Peso",
        units_per_usd_micros: 17454000,
    },
    FxRate {
        country: "Micronesia",
        currency: "U.S. Dollar",
        currency_code: "USD",
        description: "Micronesia-U.S. Dollar",
        units_per_usd_micros: 1000000,
    },
    FxRate {
        country: "Moldova",
        currency: "Leu",
        currency_code: "MDL",
        description: "Moldova-Leu",
        units_per_usd_micros: 17510000,
    },
    FxRate {
        country: "Mongolia",
        currency: "Tugrik",
        currency_code: "MNT",
        description: "Mongolia-Tugrik",
        units_per_usd_micros: 3579000000,
    },
    FxRate {
        country: "Morocco",
        currency: "Dirham",
        currency_code: "MAD",
        description: "Morocco-Dirham",
        units_per_usd_micros: 9375000,
    },
    FxRate {
        country: "Mozambique",
        currency: "Metical",
        currency_code: "MZN",
        description: "Mozambique-Metical",
        units_per_usd_micros: 63300000,
    },
    FxRate {
        country: "Myanmar",
        currency: "Kyat",
        currency_code: "MMK",
        description: "Myanmar-Kyat",
        units_per_usd_micros: 3668000000,
    },
    FxRate {
        country: "Namibia",
        currency: "Dollar",
        currency_code: "NAD",
        description: "Namibia-Dollar",
        units_per_usd_micros: 16368000,
    },
    FxRate {
        country: "Nepal",
        currency: "Rupee",
        currency_code: "NPR",
        description: "Nepal-Rupee",
        units_per_usd_micros: 151460000,
    },
    FxRate {
        country: "New Zealand",
        currency: "Dollar",
        currency_code: "NZD",
        description: "New Zealand-Dollar",
        units_per_usd_micros: 1767000,
    },
    FxRate {
        country: "Nicaragua",
        currency: "Cordoba",
        currency_code: "NIO",
        description: "Nicaragua-Cordoba",
        units_per_usd_micros: 36600000,
    },
    FxRate {
        country: "Niger",
        currency: "Cfa Franc",
        currency_code: "XOF",
        description: "Niger-Cfa Franc",
        units_per_usd_micros: 571500000,
    },
    FxRate {
        country: "Nigeria",
        currency: "Naira",
        currency_code: "NGN",
        description: "Nigeria-Naira",
        units_per_usd_micros: 1380080000,
    },
    FxRate {
        country: "Norway",
        currency: "Krone",
        currency_code: "NOK",
        description: "Norway-Krone",
        units_per_usd_micros: 9916000,
    },
    FxRate {
        country: "Oman",
        currency: "Rial",
        currency_code: "OMR",
        description: "Oman-Rial",
        units_per_usd_micros: 385000,
    },
    FxRate {
        country: "Pakistan",
        currency: "Rupee",
        currency_code: "PKR",
        description: "Pakistan-Rupee",
        units_per_usd_micros: 277700000,
    },
    FxRate {
        country: "Palau",
        currency: "Dollar",
        currency_code: "USD",
        description: "Palau-Dollar",
        units_per_usd_micros: 1000000,
    },
    FxRate {
        country: "Panama",
        currency: "Dolares",
        currency_code: "USD",
        description: "Panama-Dolares",
        units_per_usd_micros: 1000000,
    },
    FxRate {
        country: "Papua New Guinea",
        currency: "Kina",
        currency_code: "PGK",
        description: "Papua New Guinea-Kina",
        units_per_usd_micros: 4228000,
    },
    FxRate {
        country: "Paraguay",
        currency: "Guarani",
        currency_code: "PYG",
        description: "Paraguay-Guarani",
        units_per_usd_micros: 6069320000,
    },
    FxRate {
        country: "Peru",
        currency: "Sol",
        currency_code: "PEN",
        description: "Peru-Sol",
        units_per_usd_micros: 3414000,
    },
    FxRate {
        country: "Philippines",
        currency: "Peso",
        currency_code: "PHP",
        description: "Philippines-Peso",
        units_per_usd_micros: 61322000,
    },
    FxRate {
        country: "Poland",
        currency: "Zloty",
        currency_code: "PLN",
        description: "Poland-Zloty",
        units_per_usd_micros: 3768000,
    },
    FxRate {
        country: "Qatar",
        currency: "Riyal",
        currency_code: "QAR",
        description: "Qatar-Riyal",
        units_per_usd_micros: 3640000,
    },
    FxRate {
        country: "Republic Of North Macedonia",
        currency: "Denar",
        currency_code: "MKD",
        description: "Republic Of North Macedonia-Denar",
        units_per_usd_micros: 53860000,
    },
    FxRate {
        country: "Romania",
        currency: "New Leu",
        currency_code: "RON",
        description: "Romania-New Leu",
        units_per_usd_micros: 4600000,
    },
    FxRate {
        country: "Russia",
        currency: "Ruble",
        currency_code: "RUB",
        description: "Russia-Ruble",
        units_per_usd_micros: 78500000,
    },
    FxRate {
        country: "Rwanda",
        currency: "Franc",
        currency_code: "RWF",
        description: "Rwanda-Franc",
        units_per_usd_micros: 1450000000,
    },
    FxRate {
        country: "Sao Tome & Principe",
        currency: "New Dobras",
        currency_code: "STN",
        description: "Sao Tome & Principe-New Dobras",
        units_per_usd_micros: 21501000,
    },
    FxRate {
        country: "Saudi Arabia",
        currency: "Riyal",
        currency_code: "SAR",
        description: "Saudi Arabia-Riyal",
        units_per_usd_micros: 3750000,
    },
    FxRate {
        country: "Senegal",
        currency: "Cfa Franc",
        currency_code: "XOF",
        description: "Senegal-Cfa Franc",
        units_per_usd_micros: 571500000,
    },
    FxRate {
        country: "Serbia",
        currency: "Dinar",
        currency_code: "RSD",
        description: "Serbia-Dinar",
        units_per_usd_micros: 102890000,
    },
    FxRate {
        country: "Seychelles",
        currency: "Rupee",
        currency_code: "SCR",
        description: "Seychelles-Rupee",
        units_per_usd_micros: 13756000,
    },
    FxRate {
        country: "Sierra Leone",
        currency: "Leone",
        currency_code: "SLE",
        description: "Sierra Leone-Leone",
        units_per_usd_micros: 24600000,
    },
    FxRate {
        country: "Singapore",
        currency: "Dollar",
        currency_code: "SGD",
        description: "Singapore-Dollar",
        units_per_usd_micros: 1295000,
    },
    FxRate {
        country: "Solomon Islands",
        currency: "Dollar",
        currency_code: "SBD",
        description: "Solomon Islands-Dollar",
        units_per_usd_micros: 7800000,
    },
    FxRate {
        country: "Somali",
        currency: "Shilling",
        currency_code: "SOS",
        description: "Somali-Shilling",
        units_per_usd_micros: 568000000,
    },
    FxRate {
        country: "South Africa",
        currency: "Rand",
        currency_code: "ZAR",
        description: "South Africa-Rand",
        units_per_usd_micros: 16368000,
    },
    FxRate {
        country: "South Sudan",
        currency: "Sudanese Pound",
        currency_code: "SSP",
        description: "South Sudan-Sudanese Pound",
        units_per_usd_micros: 4800000000,
    },
    FxRate {
        country: "Sri Lanka",
        currency: "Rupee",
        currency_code: "LKR",
        description: "Sri Lanka-Rupee",
        units_per_usd_micros: 336000000,
    },
    FxRate {
        country: "St. Lucia",
        currency: "East Caribbean Dollar",
        currency_code: "XCD",
        description: "St. Lucia-East Caribbean Dollar",
        units_per_usd_micros: 2700000,
    },
    FxRate {
        country: "Sudan",
        currency: "Pound",
        currency_code: "SDG",
        description: "Sudan-Pound",
        units_per_usd_micros: 3550000000,
    },
    FxRate {
        country: "Suriname",
        currency: "Dollar",
        currency_code: "SRD",
        description: "Suriname-Dollar",
        units_per_usd_micros: 37325000,
    },
    FxRate {
        country: "Sweden",
        currency: "Krona",
        currency_code: "SEK",
        description: "Sweden-Krona",
        units_per_usd_micros: 9725000,
    },
    FxRate {
        country: "Switzerland",
        currency: "Franc",
        currency_code: "CHF",
        description: "Switzerland-Franc",
        units_per_usd_micros: 809000,
    },
    FxRate {
        country: "Syria",
        currency: "Pound",
        currency_code: "SYP",
        description: "Syria-Pound",
        units_per_usd_micros: 112500000,
    },
    FxRate {
        country: "Taiwan",
        currency: "Dollar",
        currency_code: "TWD",
        description: "Taiwan-Dollar",
        units_per_usd_micros: 31834000,
    },
    FxRate {
        country: "Tajikistan",
        currency: "Somoni",
        currency_code: "TJS",
        description: "Tajikistan-Somoni",
        units_per_usd_micros: 9220000,
    },
    FxRate {
        country: "Tanzania",
        currency: "Shilling",
        currency_code: "TZS",
        description: "Tanzania-Shilling",
        units_per_usd_micros: 2610000000,
    },
    FxRate {
        country: "Thailand",
        currency: "Baht",
        currency_code: "THB",
        description: "Thailand-Baht",
        units_per_usd_micros: 33200000,
    },
    FxRate {
        country: "Timor",
        currency: "Leste-Dili",
        currency_code: "USD",
        description: "Timor-Leste-Dili",
        units_per_usd_micros: 1000000,
    },
    FxRate {
        country: "Togo",
        currency: "Cfa Franc",
        currency_code: "XOF",
        description: "Togo-Cfa Franc",
        units_per_usd_micros: 571500000,
    },
    FxRate {
        country: "Tonga",
        currency: "Pa'Anga",
        currency_code: "TOP",
        description: "Tonga-Pa'Anga",
        units_per_usd_micros: 2327000,
    },
    FxRate {
        country: "Trinidad & Tobago",
        currency: "Dollar",
        currency_code: "TTD",
        description: "Trinidad & Tobago-Dollar",
        units_per_usd_micros: 6749000,
    },
    FxRate {
        country: "Tunisia",
        currency: "Dinar",
        currency_code: "TND",
        description: "Tunisia-Dinar",
        units_per_usd_micros: 2934000,
    },
    FxRate {
        country: "Turkey",
        currency: "New Lira",
        currency_code: "TRY",
        description: "Turkey-New Lira",
        units_per_usd_micros: 46656000,
    },
    FxRate {
        country: "Turkmenistan",
        currency: "New Manat",
        currency_code: "TMT",
        description: "Turkmenistan-New Manat",
        units_per_usd_micros: 3491000,
    },
    FxRate {
        country: "Uganda",
        currency: "Shilling",
        currency_code: "UGX",
        description: "Uganda-Shilling",
        units_per_usd_micros: 3660000000,
    },
    FxRate {
        country: "Ukraine",
        currency: "Hryvnia",
        currency_code: "UAH",
        description: "Ukraine-Hryvnia",
        units_per_usd_micros: 44735000,
    },
    FxRate {
        country: "United Arab Emirates",
        currency: "Dirham",
        currency_code: "AED",
        description: "United Arab Emirates-Dirham",
        units_per_usd_micros: 3673000,
    },
    FxRate {
        country: "United Kingdom",
        currency: "Pound",
        currency_code: "GBP",
        description: "United Kingdom-Pound",
        units_per_usd_micros: 756000,
    },
    FxRate {
        country: "Uruguay",
        currency: "Peso",
        currency_code: "UYU",
        description: "Uruguay-Peso",
        units_per_usd_micros: 40120000,
    },
    FxRate {
        country: "Uzbekistan",
        currency: "Som",
        currency_code: "UZS",
        description: "Uzbekistan-Som",
        units_per_usd_micros: 12000000000,
    },
    FxRate {
        country: "Vanuatu",
        currency: "Vatu",
        currency_code: "VUV",
        description: "Vanuatu-Vatu",
        units_per_usd_micros: 118300000,
    },
    FxRate {
        country: "Venezuela",
        currency: "Bolivar Soberano",
        currency_code: "VES",
        description: "Venezuela-Bolivar Soberano",
        units_per_usd_micros: 621465000,
    },
    FxRate {
        country: "Vietnam",
        currency: "Dong",
        currency_code: "VND",
        description: "Vietnam-Dong",
        units_per_usd_micros: 26255000000,
    },
    FxRate {
        country: "Western Samoa",
        currency: "Tala",
        currency_code: "WST",
        description: "Western Samoa-Tala",
        units_per_usd_micros: 2672000,
    },
    FxRate {
        country: "Yemen",
        currency: "Rial",
        currency_code: "YER",
        description: "Yemen-Rial",
        units_per_usd_micros: 528000000,
    },
    FxRate {
        country: "Zambia",
        currency: "New Kwacha",
        currency_code: "ZMW",
        description: "Zambia-New Kwacha",
        units_per_usd_micros: 18000000,
    },
    FxRate {
        country: "Zimbabwe",
        currency: "Gold",
        currency_code: "ZWG",
        description: "Zimbabwe-Gold",
        units_per_usd_micros: 25833000,
    },
];

pub fn published_fx_rates() -> Vec<FxRate> {
    RATES.to_vec()
}

fn norm(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

pub fn find_rate(query: &str) -> Result<&'static FxRate, FxError> {
    let q = norm(query);
    if q.is_empty() {
        return Err(FxError::UnknownMarket {
            query: query.to_string(),
        });
    }
    if let Some(row) = RATES.iter().find(|row| norm(row.description) == q) {
        return Ok(row);
    }
    if let Some(row) = RATES.iter().find(|row| norm(row.country) == q) {
        return Ok(row);
    }
    if let Some(row) = RATES.iter().find(|row| norm(row.currency_code) == q) {
        return Ok(row);
    }
    Err(FxError::UnknownMarket {
        query: query.to_string(),
    })
}

pub fn format_units_per_usd(micros: u128) -> String {
    let whole = micros / FX_RATE_SCALE;
    let frac = micros % FX_RATE_SCALE;
    let mut s = format!("{whole}.{frac:06}");
    while s.ends_with('0') && s.contains('.') {
        s.pop();
    }
    if s.ends_with('.') {
        s.push('0');
    }
    s
}

pub fn format_amount(nanos: u128, code: &str) -> String {
    let whole = nanos / 1_000_000_000;
    let frac = nanos % 1_000_000_000;
    let mut s = format!("{whole}.{frac:09}");
    while s.ends_with('0') && s.contains('.') {
        s.pop();
    }
    if s.ends_with('.') {
        s.push('0');
    }
    format!("{s} {code}")
}

pub fn convert_usd_nanos(usd_nanos: u128, query: &str) -> Result<FxQuote, FxError> {
    let row = find_rate(query)?;
    let amount_nanos = usd_nanos.saturating_mul(row.units_per_usd_micros) / FX_RATE_SCALE;
    Ok(FxQuote {
        country: row.country,
        currency: row.currency,
        currency_code: row.currency_code,
        units_per_usd: format_units_per_usd(row.units_per_usd_micros),
        amount: format_amount(amount_nanos, row.currency_code),
        source: FX_SOURCE,
        as_of: FX_AS_OF,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn united_states_is_one() {
        let q = convert_usd_nanos(4_000_000_000, "United States").unwrap();
        assert_eq!(q.currency_code, "USD");
        assert_eq!(q.amount, "4.0 USD");
    }

    #[test]
    fn japan_uses_treasury_yen() {
        let q = convert_usd_nanos(1_000_000_000, "Japan").unwrap();
        assert_eq!(q.currency_code, "JPY");
        assert_eq!(q.units_per_usd, "162.38");
        assert_eq!(q.amount, "162.38 JPY");
    }

    #[test]
    fn germany_uses_euro_zone_rate() {
        let q = convert_usd_nanos(1_000_000_000, "Germany").unwrap();
        assert_eq!(q.currency_code, "EUR");
        assert_eq!(q.units_per_usd, "0.877");
    }

    #[test]
    fn unknown_market_fails_closed() {
        assert!(matches!(
            convert_usd_nanos(1, "Narnia"),
            Err(FxError::UnknownMarket { .. })
        ));
    }
}
