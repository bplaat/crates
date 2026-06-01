/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! A command-line tool for looking up IP address information in a MaxMind DB file.
//!
//! Usage: maxminddb-lookup <path-to.mmdb> <ip-address>

use std::net::IpAddr;
use std::process;

use maxminddb::{Reader, geoip2};

fn main() {
    let mut args = std::env::args().skip(1);
    let db_path = args.next().unwrap_or_else(|| {
        eprintln!("Usage: lookup <path-to.mmdb> <ip-address>");
        process::exit(1);
    });
    let ip_str = args.next().unwrap_or_else(|| {
        eprintln!("Usage: lookup <path-to.mmdb> <ip-address>");
        process::exit(1);
    });

    let ip: IpAddr = ip_str.parse().unwrap_or_else(|_| {
        eprintln!("Error: '{ip_str}' is not a valid IP address");
        process::exit(1);
    });

    let reader = Reader::open_readfile(&db_path).unwrap_or_else(|e| {
        eprintln!("Error: failed to open '{db_path}': {e}");
        process::exit(1);
    });

    println!("Database:   {}", reader.metadata.database_type);
    println!("IP address: {ip}");
    println!();

    let result = reader.lookup(ip).unwrap_or_else(|e| {
        eprintln!("Error: lookup failed: {e}");
        process::exit(1);
    });

    if !result.has_data() {
        println!("No data found for this IP address.");
        return;
    }

    let city = result
        .decode::<geoip2::City>()
        .unwrap_or_else(|e| {
            eprintln!("Error: failed to decode record: {e}");
            process::exit(1);
        })
        .expect("has_data was true but decode returned None");

    if let Some(code) = &city.continent.code {
        println!(
            "Continent:       {} ({})",
            city.continent.names.english.as_deref().unwrap_or("?"),
            code
        );
    }
    if let Some(code) = &city.country.iso_code {
        let eu = if city.country.is_in_european_union == Some(true) {
            " [EU]"
        } else {
            ""
        };
        println!(
            "Country:         {} ({}){}",
            city.country.names.english.as_deref().unwrap_or("?"),
            code,
            eu
        );
    }
    if let Some(code) = &city.registered_country.iso_code {
        println!(
            "Registered in:   {} ({})",
            city.registered_country
                .names
                .english
                .as_deref()
                .unwrap_or("?"),
            code
        );
    }
    for (i, sub) in city.subdivisions.iter().enumerate() {
        println!(
            "Subdivision {}:   {} ({})",
            i + 1,
            sub.names.english.as_deref().unwrap_or("?"),
            sub.iso_code.as_deref().unwrap_or("?")
        );
    }
    if let Some(name) = &city.city.names.english {
        println!("City:            {name}");
    }
    if let Some(code) = &city.postal.code {
        println!("Postal code:     {code}");
    }
    if let (Some(lat), Some(lon)) = (city.location.latitude, city.location.longitude) {
        println!("Coordinates:     {lat:.4}, {lon:.4}");
    }
    if let Some(r) = city.location.accuracy_radius {
        println!("Accuracy radius: {r} km");
    }
    if let Some(tz) = &city.location.time_zone {
        println!("Time zone:       {tz}");
    }
}
