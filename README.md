# Bassie's Rust crates

A collection of minimal Rust crates that I created for myself

## Crates

-   [base64](lib/base64) A base64 encoder and decoder
-   [getrandom](lib/getrandom) A minimal crypto random bytes library
-   [http](lib/http) A simple HTTP/1.1 server/client library
-   [objc](lib/objc) A basic Objective-C ffi library
-   [openapi-generator](lib/openapi-generator) A simple OpenAPI code generator
-   [pbkdf2](lib/pbkdf2) A unsecure PBKDF2-HMAC-SHA256 password hashing library
-   [router](lib/router) A simple router for HTTP library
-   [sqlite](lib/sqlite) A SQLite Rust library
-   [sqlite_derive](lib/sqlite_derive) SQLite derive macro's library
-   [sqlite3-sys](lib/sqlite3-sys) A SQLite3 sys bindings library
-   [threadpool](lib/threadpool) A very basic thread pool library
-   [url](lib/url) A minimal URL parser library
-   [useragent](lib/useragent) A basic user agent parser library
-   [uuid](lib/uuid) A minimal UUID library
-   [validate](lib/validate) A simple struct validation library
-   [validate_derive](lib/validate_derive) Validation derive macro's library
-   [webview](lib/webview) A simple webview library

## TODO

-   [ ] webview: Add Windows (win32 + Webview2) support
-   [ ] http: keep-alive requests
-   [ ] http: multipart/form-data
-   [ ] http: Chunked transfer encoding
-   [ ] openapi-generator: Add TypeScript client generation
-   [ ] webview: Add Linux (Gtk + Webkit2Gtk) support
