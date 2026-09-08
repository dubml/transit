#!/usr/bin/env python3
"""Generate crates/core/src/fx.rs from U.S. Treasury Reporting Rates of Exchange."""

import json
import urllib.request
from pathlib import Path

URL = (
    "https://api.fiscaldata.treasury.gov/services/api/fiscal_service"
    "/v1/accounting/od/rates_of_exchange"
    "?fields=country,currency,country_currency_desc,exchange_rate,record_date"
    "&filter=record_date:eq:2026-06-30&page[size]=500"
)

COUNTRY_CCY = {
    "Afghanistan": "AFN",
    "Albania": "ALL",
    "Algeria": "DZD",
    "Angola": "AOA",
    "Antigua & Barbuda": "XCD",
    "Argentina": "ARS",
    "Armenia": "AMD",
    "Australia": "AUD",
    "Azerbaijan": "AZN",
    "Bahamas": "BSD",
    "Bahrain": "BHD",
    "Bangladesh": "BDT",
    "Barbados": "BBD",
    "Belarus": "BYN",
    "Belize": "BZD",
    "Benin": "XOF",
    "Bermuda": "BMD",
    "Bolivia": "BOB",
    "Bosnia": "BAM",
    "Botswana": "BWP",
    "Brazil": "BRL",
    "Brunei": "BND",
    "Bulgaria": "BGN",
    "Burkina Faso": "XOF",
    "Burundi": "BIF",
    "Cambodia": "KHR",
    "Cameroon": "XAF",
    "Canada": "CAD",
    "Cape Verde": "CVE",
    "Cayman Islands": "KYD",
    "Central African Republic": "XAF",
    "Chad": "XAF",
    "Chile": "CLP",
    "China": "CNY",
    "Colombia": "COP",
    "Comoros": "KMF",
    "Congo": "XAF",
    "Costa Rica": "CRC",
    "Cote D'Ivoire": "XOF",
    "Cuba": "CUP",
    "Curacao": "XCG",
    "Cyprus": "EUR",
    "Czech Republic": "CZK",
    "Democratic Republic Of Congo": "CDF",
    "Denmark": "DKK",
    "Djibouti": "DJF",
    "Dominican Republic": "DOP",
    "Ecuador": "USD",
    "Egypt": "EGP",
    "El Salvador": "USD",
    "Equatorial Guinea": "XAF",
    "Eritrea": "ERN",
    "Eswatini": "SZL",
    "Ethiopia": "ETB",
    "Euro Zone": "EUR",
    "Fiji": "FJD",
    "Gabon": "XAF",
    "Gambia": "GMD",
    "Georgia": "GEL",
    "Ghana": "GHS",
    "Grenada": "XCD",
    "Guatemala": "GTQ",
    "Guinea": "GNF",
    "Guinea Bissau": "XOF",
    "Guyana": "GYD",
    "Haiti": "HTG",
    "Honduras": "HNL",
    "Hong Kong": "HKD",
    "Hungary": "HUF",
    "Iceland": "ISK",
    "India": "INR",
    "Indonesia": "IDR",
    "Iran": "IRR",
    "Iraq": "IQD",
    "Israel": "ILS",
    "Jamaica": "JMD",
    "Japan": "JPY",
    "Jordan": "JOD",
    "Kazakhstan": "KZT",
    "Kenya": "KES",
    "Korea": "KRW",
    "Kuwait": "KWD",
    "Kyrgyzstan": "KGS",
    "Laos": "LAK",
    "Lebanon": "LBP",
    "Lesotho": "LSL",
    "Liberia": "LRD",
    "Libya": "LYD",
    "Madagascar": "MGA",
    "Malawi": "MWK",
    "Malaysia": "MYR",
    "Maldives": "MVR",
    "Mali": "XOF",
    "Marshall Islands": "USD",
    "Mauritania": "MRU",
    "Mauritius": "MUR",
    "Mexico": "MXN",
    "Micronesia": "USD",
    "Moldova": "MDL",
    "Mongolia": "MNT",
    "Morocco": "MAD",
    "Mozambique": "MZN",
    "Myanmar": "MMK",
    "Namibia": "NAD",
    "Nepal": "NPR",
    "New Zealand": "NZD",
    "Nicaragua": "NIO",
    "Niger": "XOF",
    "Nigeria": "NGN",
    "Norway": "NOK",
    "Oman": "OMR",
    "Pakistan": "PKR",
    "Palau": "USD",
    "Panama": "USD",
    "Papua New Guinea": "PGK",
    "Paraguay": "PYG",
    "Peru": "PEN",
    "Philippines": "PHP",
    "Poland": "PLN",
    "Qatar": "QAR",
    "Republic Of North Macedonia": "MKD",
    "Romania": "RON",
    "Russia": "RUB",
    "Rwanda": "RWF",
    "Sao Tome & Principe": "STN",
    "Saudi Arabia": "SAR",
    "Senegal": "XOF",
    "Serbia": "RSD",
    "Seychelles": "SCR",
    "Sierra Leone": "SLE",
    "Singapore": "SGD",
    "Solomon Islands": "SBD",
    "Somali": "SOS",
    "South Africa": "ZAR",
    "South Sudan": "SSP",
    "Sri Lanka": "LKR",
    "St. Lucia": "XCD",
    "Sudan": "SDG",
    "Suriname": "SRD",
    "Sweden": "SEK",
    "Switzerland": "CHF",
    "Syria": "SYP",
    "Taiwan": "TWD",
    "Tajikistan": "TJS",
    "Tanzania": "TZS",
    "Thailand": "THB",
    "Timor": "USD",
    "Togo": "XOF",
    "Tonga": "TOP",
    "Trinidad & Tobago": "TTD",
    "Tunisia": "TND",
    "Turkey": "TRY",
    "Turkmenistan": "TMT",
    "Uganda": "UGX",
    "Ukraine": "UAH",
    "United Arab Emirates": "AED",
    "United Kingdom": "GBP",
    "United States": "USD",
    "Uruguay": "UYU",
    "Uzbekistan": "UZS",
    "Vanuatu": "VUV",
    "Venezuela": "VES",
    "Vietnam": "VND",
    "Western Samoa": "WST",
    "Yemen": "YER",
    "Zambia": "ZMW",
    "Zimbabwe": "ZWG",
    "Austria": "EUR",
    "Belgium": "EUR",
    "Croatia": "EUR",
    "Estonia": "EUR",
    "Finland": "EUR",
    "France": "EUR",
    "Germany": "EUR",
    "Greece": "EUR",
    "Ireland": "EUR",
    "Italy": "EUR",
    "Latvia": "EUR",
    "Lithuania": "EUR",
    "Luxembourg": "EUR",
    "Malta": "EUR",
    "Netherlands": "EUR",
    "Portugal": "EUR",
    "Slovakia": "EUR",
    "Slovenia": "EUR",
    "Spain": "EUR",
}


def rust_str(value: str) -> str:
    return value.replace("\\", "\\\\").replace('"', '\\"')


def rate_micros(value: str) -> int:
    if "." in value:
        whole, frac = value.split(".", 1)
        frac = (frac + "000000")[:6]
        return int(whole) * 1_000_000 + int(frac)
    return int(value) * 1_000_000


def main() -> None:
    payload = json.load(urllib.request.urlopen(URL, timeout=30))
    seen = set()
    rows = []
    for row in payload["data"]:
        key = row["country_currency_desc"]
        if key in seen:
            continue
        seen.add(key)
        rows.append(row)

    euro = next(row for row in rows if row["country"] == "Euro Zone")
    extras = [
        {
            "country": "United States",
            "currency": "Dollar",
            "country_currency_desc": "United States-Dollar",
            "exchange_rate": "1.0",
        }
    ]
    for country in [
        "Austria",
        "Belgium",
        "Croatia",
        "Estonia",
        "Finland",
        "France",
        "Germany",
        "Greece",
        "Ireland",
        "Italy",
        "Latvia",
        "Lithuania",
        "Luxembourg",
        "Malta",
        "Netherlands",
        "Portugal",
        "Slovakia",
        "Slovenia",
        "Spain",
    ]:
        extras.append(
            {
                "country": country,
                "currency": "Euro",
                "country_currency_desc": f"{country}-Euro",
                "exchange_rate": euro["exchange_rate"],
            }
        )

    all_rows = extras + rows
    out = []
    a = out.append
    a("//! Published FX: U.S. Treasury Reporting Rates of Exchange.")
    a("//! Units of foreign currency per 1 USD. Integer micro-units (1e-6).")
    a("//! Unknown country/currency fails closed. No invented rates.")
    a("")
    a("use serde::Serialize;")
    a("")
    a("pub const FX_SOURCE: &str =")
    a('    "https://fiscaldata.treasury.gov/datasets/treasury-reporting-rates-exchange/";')
    a('pub const FX_AS_OF: &str = "2026-06-30";')
    a("pub const FX_RATE_SCALE: u128 = 1_000_000;")
    a("")
    a("#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]")
    a("pub struct FxRate {")
    a("    pub country: &'static str,")
    a("    pub currency: &'static str,")
    a("    pub currency_code: &'static str,")
    a("    pub description: &'static str,")
    a("    /// Foreign currency micro-units per 1 USD.")
    a("    pub units_per_usd_micros: u128,")
    a("}")
    a("")
    a("#[derive(Debug, Clone, PartialEq, Eq)]")
    a("pub enum FxError {")
    a("    UnknownMarket { query: String },")
    a("}")
    a("")
    a("impl std::fmt::Display for FxError {")
    a("    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {")
    a("        match self {")
    a("            Self::UnknownMarket { query } => {")
    a('                write!(f, "no published Treasury FX row for {query}")')
    a("            }")
    a("        }")
    a("    }")
    a("}")
    a("impl std::error::Error for FxError {}")
    a("")
    a("#[derive(Debug, Clone, PartialEq, Eq, Serialize)]")
    a("pub struct FxQuote {")
    a("    pub country: &'static str,")
    a("    pub currency: &'static str,")
    a("    pub currency_code: &'static str,")
    a("    pub units_per_usd: String,")
    a("    pub amount: String,")
    a("    pub source: &'static str,")
    a("    pub as_of: &'static str,")
    a("}")
    a("")
    a("const RATES: &[FxRate] = &[")
    for row in all_rows:
        country = row["country"]
        code = COUNTRY_CCY.get(country, "XXX")
        micros = rate_micros(row["exchange_rate"])
        a("    FxRate {")
        a(f'        country: "{rust_str(country)}",')
        a(f'        currency: "{rust_str(row["currency"])}",')
        a(f'        currency_code: "{code}",')
        a(f'        description: "{rust_str(row["country_currency_desc"])}",')
        a(f"        units_per_usd_micros: {micros},")
        a("    },")
    a("];")
    a("")
    a("pub fn published_fx_rates() -> Vec<FxRate> {")
    a("    RATES.to_vec()")
    a("}")
    a("")
    a("fn norm(value: &str) -> String {")
    a("    value.trim().to_ascii_lowercase()")
    a("}")
    a("")
    a("pub fn find_rate(query: &str) -> Result<&'static FxRate, FxError> {")
    a("    let q = norm(query);")
    a("    if q.is_empty() {")
    a("        return Err(FxError::UnknownMarket {")
    a("            query: query.to_string(),")
    a("        });")
    a("    }")
    a("    if let Some(row) = RATES.iter().find(|row| norm(row.description) == q) {")
    a("        return Ok(row);")
    a("    }")
    a("    if let Some(row) = RATES.iter().find(|row| norm(row.country) == q) {")
    a("        return Ok(row);")
    a("    }")
    a("    if let Some(row) = RATES.iter().find(|row| norm(row.currency_code) == q) {")
    a("        return Ok(row);")
    a("    }")
    a("    Err(FxError::UnknownMarket {")
    a("        query: query.to_string(),")
    a("    })")
    a("}")
    a("")
    a("pub fn format_units_per_usd(micros: u128) -> String {")
    a("    let whole = micros / FX_RATE_SCALE;")
    a("    let frac = micros % FX_RATE_SCALE;")
    a('    let mut s = format!("{whole}.{frac:06}");')
    a("    while s.ends_with('0') && s.contains('.') {")
    a("        s.pop();")
    a("    }")
    a("    if s.ends_with('.') {")
    a("        s.push('0');")
    a("    }")
    a("    s")
    a("}")
    a("")
    a("pub fn format_amount(nanos: u128, code: &str) -> String {")
    a("    let whole = nanos / 1_000_000_000;")
    a("    let frac = nanos % 1_000_000_000;")
    a('    let mut s = format!("{whole}.{frac:09}");')
    a("    while s.ends_with('0') && s.contains('.') {")
    a("        s.pop();")
    a("    }")
    a("    if s.ends_with('.') {")
    a("        s.push('0');")
    a("    }")
    a('    format!("{s} {code}")')
    a("}")
    a("")
    a("pub fn convert_usd_nanos(usd_nanos: u128, query: &str) -> Result<FxQuote, FxError> {")
    a("    let row = find_rate(query)?;")
    a("    let amount_nanos = usd_nanos.saturating_mul(row.units_per_usd_micros) / FX_RATE_SCALE;")
    a("    Ok(FxQuote {")
    a("        country: row.country,")
    a("        currency: row.currency,")
    a("        currency_code: row.currency_code,")
    a("        units_per_usd: format_units_per_usd(row.units_per_usd_micros),")
    a("        amount: format_amount(amount_nanos, row.currency_code),")
    a("        source: FX_SOURCE,")
    a("        as_of: FX_AS_OF,")
    a("    })")
    a("}")
    a("")
    a("#[cfg(test)]")
    a("mod tests {")
    a("    use super::*;")
    a("")
    a("    #[test]")
    a("    fn united_states_is_one() {")
    a("        let q = convert_usd_nanos(4_000_000_000, \"United States\").unwrap();")
    a("        assert_eq!(q.currency_code, \"USD\");")
    a("        assert_eq!(q.amount, \"4.0 USD\");")
    a("    }")
    a("")
    a("    #[test]")
    a("    fn japan_uses_treasury_yen() {")
    a("        let q = convert_usd_nanos(1_000_000_000, \"Japan\").unwrap();")
    a("        assert_eq!(q.currency_code, \"JPY\");")
    a("        assert_eq!(q.units_per_usd, \"162.38\");")
    a("        assert_eq!(q.amount, \"162.38 JPY\");")
    a("    }")
    a("")
    a("    #[test]")
    a("    fn germany_uses_euro_zone_rate() {")
    a("        let q = convert_usd_nanos(1_000_000_000, \"Germany\").unwrap();")
    a("        assert_eq!(q.currency_code, \"EUR\");")
    a("        assert_eq!(q.units_per_usd, \"0.877\");")
    a("    }")
    a("")
    a("    #[test]")
    a("    fn unknown_market_fails_closed() {")
    a("        assert!(matches!(")
    a("            convert_usd_nanos(1, \"Narnia\"),")
    a("            Err(FxError::UnknownMarket { .. })")
    a("        ));")
    a("    }")
    a("}")
    a("")

    dest = Path(__file__).resolve().parents[1] / "crates/core/src/fx.rs"
    dest.write_text("\n".join(out), encoding="utf-8")
    print(f"wrote {dest} rows={len(all_rows)}")


if __name__ == "__main__":
    main()
