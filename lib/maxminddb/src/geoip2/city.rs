/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! City database model structs.

use std::collections::HashMap;

use serde::Deserialize;

// MARK: Names

/// Localized names for a geographic entity, keyed by locale code.
#[derive(Debug, Default, Deserialize)]
pub struct Names {
    /// English name.
    #[serde(rename = "en")]
    pub english: Option<String>,
    /// German name.
    #[serde(rename = "de")]
    pub de: Option<String>,
    /// Spanish name.
    #[serde(rename = "es")]
    pub es: Option<String>,
    /// French name.
    #[serde(rename = "fr")]
    pub fr: Option<String>,
    /// Japanese name.
    #[serde(rename = "ja")]
    pub ja: Option<String>,
    /// Brazilian Portuguese name.
    #[serde(rename = "pt-BR")]
    pub pt_br: Option<String>,
    /// Russian name.
    #[serde(rename = "ru")]
    pub ru: Option<String>,
    /// Simplified Chinese name.
    #[serde(rename = "zh-CN")]
    pub zh_cn: Option<String>,
}

impl Names {
    /// Return the name in the given locale, falling back to English.
    pub fn get(&self, locale: &str) -> Option<&str> {
        match locale {
            "en" => self.english.as_deref(),
            "de" => self.de.as_deref(),
            "es" => self.es.as_deref(),
            "fr" => self.fr.as_deref(),
            "ja" => self.ja.as_deref(),
            "pt-BR" => self.pt_br.as_deref(),
            "ru" => self.ru.as_deref(),
            "zh-CN" => self.zh_cn.as_deref(),
            _ => None,
        }
    }
}

// MARK: Continent

/// Continent data for an IP address.
#[derive(Debug, Default, Deserialize)]
pub struct Continent {
    /// Two-letter continent code (e.g. "EU", "NA").
    pub code: Option<String>,
    /// Continent GeoName ID.
    pub geoname_id: Option<u32>,
    /// Localized continent names.
    #[serde(default)]
    pub names: Names,
}

// MARK: Country

/// Country data for an IP address.
#[derive(Debug, Default, Deserialize)]
pub struct Country {
    /// Two-letter ISO 3166-1 alpha-2 country code (e.g. "US", "DE").
    pub iso_code: Option<String>,
    /// Country GeoName ID.
    pub geoname_id: Option<u32>,
    /// Whether the country is in the European Union.
    pub is_in_european_union: Option<bool>,
    /// Localized country names.
    #[serde(default)]
    pub names: Names,
}

// MARK: Subdivision

/// Subdivision (state, province, etc.) data.
#[derive(Debug, Default, Deserialize)]
pub struct Subdivision {
    /// ISO 3166-2 subdivision code (e.g. "TX", "ENG").
    pub iso_code: Option<String>,
    /// Subdivision GeoName ID.
    pub geoname_id: Option<u32>,
    /// Localized subdivision names.
    #[serde(default)]
    pub names: Names,
}

// MARK: CityRecord

/// City-level data for an IP address.
#[derive(Debug, Default, Deserialize)]
pub struct CityRecord {
    /// City GeoName ID.
    pub geoname_id: Option<u32>,
    /// Localized city names.
    #[serde(default)]
    pub names: Names,
}

// MARK: Location

/// Geographic location data for an IP address.
#[derive(Debug, Default, Deserialize)]
pub struct Location {
    /// Latitude.
    pub latitude: Option<f64>,
    /// Longitude.
    pub longitude: Option<f64>,
    /// Accuracy radius in kilometers.
    pub accuracy_radius: Option<u16>,
    /// Time zone identifier (e.g. "America/Chicago").
    pub time_zone: Option<String>,
    /// Metro code (US only).
    pub metro_code: Option<u16>,
}

// MARK: Postal

/// Postal code data for an IP address.
#[derive(Debug, Default, Deserialize)]
pub struct Postal {
    /// Postal code string.
    pub code: Option<String>,
}

// MARK: RepresentedCountry

/// Country represented by users of an IP (e.g. military base or embassy).
#[derive(Debug, Default, Deserialize)]
pub struct RepresentedCountry {
    /// Two-letter ISO country code.
    pub iso_code: Option<String>,
    /// GeoName ID.
    pub geoname_id: Option<u32>,
    /// Country type (e.g. "military").
    #[serde(rename = "type")]
    pub country_type: Option<String>,
    /// Localized names.
    #[serde(default)]
    pub names: Names,
}

// MARK: Traits

/// Various traits associated with the IP address.
#[derive(Debug, Default, Deserialize)]
pub struct Traits {
    /// Whether the IP is an anycast address.
    pub is_anycast: Option<bool>,
}

// MARK: City

/// GeoIP2/GeoLite2 City database record.
#[derive(Debug, Default, Deserialize)]
pub struct City {
    /// City data.
    #[serde(default)]
    pub city: CityRecord,
    /// Continent data.
    #[serde(default)]
    pub continent: Continent,
    /// Country data.
    #[serde(default)]
    pub country: Country,
    /// Location data.
    #[serde(default)]
    pub location: Location,
    /// Postal code data.
    #[serde(default)]
    pub postal: Postal,
    /// Country where the ISP has registered the IP block.
    #[serde(default)]
    pub registered_country: Country,
    /// Country represented by users of this IP.
    #[serde(default)]
    pub represented_country: RepresentedCountry,
    /// Subdivisions ordered from largest to smallest.
    #[serde(default)]
    pub subdivisions: Vec<Subdivision>,
    /// Various traits.
    #[serde(default)]
    pub traits: Traits,
}

// MARK: Country top-level

/// GeoIP2/GeoLite2 Country database record.
#[derive(Debug, Default, Deserialize)]
pub struct CountryRecord {
    /// Continent data.
    #[serde(default)]
    pub continent: Continent,
    /// Country data.
    #[serde(default)]
    pub country: Country,
    /// Country where the ISP registered the IP.
    #[serde(default)]
    pub registered_country: Country,
    /// Country represented by this IP.
    #[serde(default)]
    pub represented_country: RepresentedCountry,
    /// Various traits.
    #[serde(default)]
    pub traits: Traits,
}

// MARK: Asn

/// GeoLite2 ASN database record.
#[derive(Debug, Default, Deserialize)]
pub struct Asn {
    /// Autonomous System Number.
    pub autonomous_system_number: Option<u32>,
    /// Autonomous System Organization.
    pub autonomous_system_organization: Option<String>,
}

// MARK: AnonymousIp

/// GeoIP2 Anonymous IP database record.
#[derive(Debug, Default, Deserialize)]
pub struct AnonymousIp {
    /// Whether this is an anonymous proxy.
    pub is_anonymous: Option<bool>,
    /// Whether this is a VPN.
    pub is_anonymous_vpn: Option<bool>,
    /// Whether this is a hosting provider.
    pub is_hosting_provider: Option<bool>,
    /// Whether this is a public proxy.
    pub is_public_proxy: Option<bool>,
    /// Whether this is a residential proxy.
    pub is_residential_proxy: Option<bool>,
    /// Whether this is a Tor exit node.
    pub is_tor_exit_node: Option<bool>,
}

// MARK: ConnectionType

/// GeoIP2 Connection-Type database record.
#[derive(Debug, Default, Deserialize)]
pub struct ConnectionType {
    /// Connection type string.
    pub connection_type: Option<String>,
}

// MARK: Domain

/// GeoIP2 Domain database record.
#[derive(Debug, Default, Deserialize)]
pub struct Domain {
    /// Domain name.
    pub domain: Option<String>,
}

// MARK: Isp

/// GeoIP2 ISP database record.
#[derive(Debug, Default, Deserialize)]
pub struct Isp {
    /// Autonomous System Number.
    pub autonomous_system_number: Option<u32>,
    /// Autonomous System Organization.
    pub autonomous_system_organization: Option<String>,
    /// ISP name.
    pub isp: Option<String>,
    /// Organization name.
    pub organization: Option<String>,
}

// MARK: Extra names alias

/// Localized names map (alias for external use).
pub type NamesMap = HashMap<String, String>;
