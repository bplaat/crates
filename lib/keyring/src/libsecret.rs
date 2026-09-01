/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(unsafe_code)]

use std::ffi::{CStr, CString, c_char, c_void};
use std::ptr::{null, null_mut};

use zeroize::Zeroizing;

use crate::{Error, Result};

const SECRET_SCHEMA_NONE: i32 = 0;
const SECRET_SCHEMA_ATTRIBUTE_STRING: i32 = 0;

#[repr(C)]
struct SecretSchema([u8; 0]);

#[repr(C)]
struct GError {
    domain: u32,
    code: i32,
    message: *mut c_char,
}

unsafe extern "C" {
    fn secret_schema_new(name: *const c_char, flags: i32, ...) -> *mut SecretSchema;
    fn secret_schema_unref(schema: *mut SecretSchema);
    fn secret_password_store_sync(
        schema: *const SecretSchema,
        collection: *const c_char,
        label: *const c_char,
        password: *const c_char,
        cancellable: *mut c_void,
        error: *mut *mut GError,
        ...
    ) -> i32;
    fn secret_password_lookup_sync(
        schema: *const SecretSchema,
        cancellable: *mut c_void,
        error: *mut *mut GError,
        ...
    ) -> *mut c_char;
    fn secret_password_clear_sync(
        schema: *const SecretSchema,
        cancellable: *mut c_void,
        error: *mut *mut GError,
        ...
    ) -> i32;
    fn secret_password_free(password: *mut c_char);
    fn g_error_free(error: *mut GError);
}

struct Schema(*mut SecretSchema);

impl Schema {
    fn new() -> Result<Self> {
        // SAFETY: all arguments are correctly typed C varargs and the list is NULL-terminated.
        let schema = unsafe {
            secret_schema_new(
                c"nl.bplaat.keyring.Password".as_ptr(),
                SECRET_SCHEMA_NONE,
                c"service".as_ptr(),
                SECRET_SCHEMA_ATTRIBUTE_STRING,
                c"account".as_ptr(),
                SECRET_SCHEMA_ATTRIBUTE_STRING,
                null::<c_char>(),
            )
        };
        if schema.is_null() {
            Err(Error::Platform(
                "failed to create libsecret schema".to_string(),
            ))
        } else {
            Ok(Self(schema))
        }
    }
}

impl Drop for Schema {
    fn drop(&mut self) {
        // SAFETY: Schema owns the reference returned by secret_schema_new.
        unsafe { secret_schema_unref(self.0) };
    }
}

pub(crate) fn set_password(service: &str, account: &str, password: &str) -> Result<()> {
    let schema = Schema::new()?;
    let service = c_string(service)?;
    let account = c_string(account)?;
    let password = Zeroizing::new(c_string(password)?);
    let mut error = null_mut();
    // SAFETY: pointers remain valid for the call, attributes match the schema, and varargs end in NULL.
    let succeeded = unsafe {
        secret_password_store_sync(
            schema.0,
            null(),
            service.as_ptr(),
            password.as_ptr(),
            null_mut(),
            &mut error,
            c"service".as_ptr(),
            service.as_ptr(),
            c"account".as_ptr(),
            account.as_ptr(),
            null::<c_char>(),
        )
    };
    if !error.is_null() {
        return Err(take_error(error, "store"));
    }
    if succeeded == 0 {
        Err(Error::Platform(
            "libsecret failed to store credential".to_string(),
        ))
    } else {
        Ok(())
    }
}

pub(crate) fn get_password(service: &str, account: &str) -> Result<String> {
    let schema = Schema::new()?;
    let service = c_string(service)?;
    let account = c_string(account)?;
    let mut error = null_mut();
    // SAFETY: pointers remain valid for the call, attributes match the schema, and varargs end in NULL.
    let password = unsafe {
        secret_password_lookup_sync(
            schema.0,
            null_mut(),
            &mut error,
            c"service".as_ptr(),
            service.as_ptr(),
            c"account".as_ptr(),
            account.as_ptr(),
            null::<c_char>(),
        )
    };
    if !error.is_null() {
        return Err(take_error(error, "load"));
    }
    if password.is_null() {
        return Err(Error::NoEntry);
    }
    // SAFETY: libsecret returned a valid NUL-terminated password string.
    let bytes = unsafe { CStr::from_ptr(password) }.to_bytes().to_vec();
    // SAFETY: password was allocated by libsecret and must be wiped and freed with this function.
    unsafe { secret_password_free(password) };
    String::from_utf8(bytes)
        .map_err(|_| Error::Platform("libsecret returned a non-UTF-8 password".to_string()))
}

pub(crate) fn delete_credential(service: &str, account: &str) -> Result<()> {
    let schema = Schema::new()?;
    let service = c_string(service)?;
    let account = c_string(account)?;
    let mut error = null_mut();
    // SAFETY: pointers remain valid for the call, attributes match the schema, and varargs end in NULL.
    let removed = unsafe {
        secret_password_clear_sync(
            schema.0,
            null_mut(),
            &mut error,
            c"service".as_ptr(),
            service.as_ptr(),
            c"account".as_ptr(),
            account.as_ptr(),
            null::<c_char>(),
        )
    };
    if !error.is_null() {
        return Err(take_error(error, "delete"));
    }
    if removed == 0 {
        Err(Error::NoEntry)
    } else {
        Ok(())
    }
}

fn c_string(value: &str) -> Result<CString> {
    CString::new(value).map_err(|_| Error::InvalidInput)
}

fn take_error(error: *mut GError, operation: &str) -> Error {
    // SAFETY: a non-NULL GError from GLib has a valid NUL-terminated message until freed.
    let message = unsafe { CStr::from_ptr((*error).message) }
        .to_string_lossy()
        .into_owned();
    // SAFETY: ownership of this GError was transferred to the caller.
    unsafe { g_error_free(error) };
    Error::Platform(format!(
        "failed to {operation} libsecret credential: {message}"
    ))
}
