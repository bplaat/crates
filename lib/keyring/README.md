# keyring

A minimal replacement for the [keyring](https://crates.io/crates/keyring) crate that wraps the operating system credential store.

- macOS: Keychain Services
- Windows: Credential Manager
- Linux and BSD: `libsecret-1` through direct C bindings

Install the `libsecret-1` runtime on Linux before building or running an application that uses this crate.

Ubuntu / Debian:

```sh
sudo apt install libsecret-1-0
```

Fedora:

```sh
sudo dnf install libsecret
```

The Linux desktop session must also provide a Secret Service implementation, such as GNOME Keyring.
Headless environments without a Secret Service can compile applications, but cannot store or load
credentials.

```rs
fn main() -> Result<(), keyring::Error> {
    let entry = keyring::Entry::new("com.example.App", "account")?;
    entry.set_password("secret")?;
    let password = entry.get_password()?;
    entry.delete_credential()?;
    Ok(())
}
```
