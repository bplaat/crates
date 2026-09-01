/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use from_derive::{FromEnum, FromStruct};

enum ExternalState {
    Ready,
    Done,
}

#[derive(FromEnum)]
#[from_enum(ExternalState)]
enum State {
    Ready,
    Done,
}

enum ExternalAccess {
    Granted,
    Denied,
}

#[derive(FromEnum)]
#[from_enum(ExternalAccess, only_from)]
enum Access {
    Granted,
    Denied,
}

impl From<Access> for ExternalAccess {
    fn from(value: Access) -> Self {
        match value {
            Access::Granted => Self::Granted,
            Access::Denied => Self::Denied,
        }
    }
}

struct ExternalRecord<T>
where
    T: Clone,
{
    value: T,
}

struct ExternalCredentials {
    password: String,
    recovery_code: Option<String>,
}

struct Secret<T>(T);

impl<T> From<T> for Secret<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

#[derive(FromStruct)]
#[from_struct(ExternalCredentials, only_from)]
struct Credentials {
    password: Secret<String>,
    recovery_code: Option<Secret<String>>,
}

#[derive(FromStruct)]
#[from_struct(ExternalRecord)]
struct Record<T>
where
    T: Clone,
{
    value: T,
}

fn main() {
    let external: ExternalState = State::Ready.into();
    let _: State = external.into();

    let access: Access = ExternalAccess::Granted.into();
    let _: ExternalAccess = access.into();

    let external: ExternalRecord<String> = Record {
        value: String::new(),
    }
    .into();
    let _: Record<String> = external.into();

    let credentials: Credentials = ExternalCredentials {
        password: String::new(),
        recovery_code: Some(String::new()),
    }
    .into();
    let _ = credentials.password.0;
    let _ = credentials.recovery_code.unwrap().0;
}
