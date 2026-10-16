/*
 * Copyright (c) 2024-2025 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::str::FromStr;

// MARK: Version
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Version {
    Http1_0,
    #[default]
    Http1_1,
}

impl FromStr for Version {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "HTTP/1.0" => Ok(Version::Http1_0),
            _ => Ok(Version::Http1_1),
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Version::Http1_0 => "HTTP/1.0",
                Version::Http1_1 => "HTTP/1.1",
            }
        )
    }
}

// MARK: Method
/// HTTP method
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Method {
    /// GET
    #[default]
    Get,
    /// HEAD
    Head,
    /// POST
    Post,
    /// PUT
    Put,
    /// DELETE
    Delete,
    /// CONNECT
    Connect,
    /// OPTIONS
    Options,
    /// TRACE
    Trace,
    /// PATCH
    Patch,
}

impl FromStr for Method {
    type Err = InvalidMethodError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::Get),
            "HEAD" => Ok(Method::Head),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "CONNECT" => Ok(Method::Connect),
            "OPTIONS" => Ok(Method::Options),
            "TRACE" => Ok(Method::Trace),
            "PATCH" => Ok(Method::Patch),
            _ => Err(InvalidMethodError),
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Method::Get => "GET",
                Method::Head => "HEAD",
                Method::Post => "POST",
                Method::Put => "PUT",
                Method::Delete => "DELETE",
                Method::Connect => "CONNECT",
                Method::Options => "OPTIONS",
                Method::Trace => "TRACE",
                Method::Patch => "PATCH",
            }
        )
    }
}

// MARK: InvalidMethodError
#[derive(Debug)]
pub struct InvalidMethodError;

impl Display for InvalidMethodError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid HTTP method")
    }
}

impl Error for InvalidMethodError {}

// MARK: Status
/// Http status
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Status {
    /// 100 Continue
    Continue = 100,
    /// 101 Switching Protocols
    SwitchingProtocols = 101,
    /// 102 Processing
    Processing = 102,
    /// 200 OK
    #[default]
    Ok = 200,
    /// 201 Created
    Created = 201,
    /// 202 Accepted
    Accepted = 202,
    /// 203 Non-Authoritative Information
    NonAuthoritativeInformation = 203,
    /// 204 No Content
    NoContent = 204,
    /// 205 Reset Content
    ResetContent = 205,
    /// 206 Partial Content
    PartialContent = 206,
    /// 207 Multi-Status
    MultiStatus = 207,
    /// 208 Already Reported
    AlreadyReported = 208,
    /// 226 IM Used
    IMUsed = 226,
    /// 300 Multiple Choices
    MultipleChoices = 300,
    /// 301 Moved Permanently
    MovedPermanently = 301,
    /// 302 Found
    Found = 302,
    /// 303 See Other
    SeeOther = 303,
    /// 304 Not Modified
    NotModified = 304,
    /// 305 Use Proxy
    UseProxy = 305,
    /// 307 Temporary Redirect
    TemporaryRedirect = 307,
    /// 308 Permanent Redirect
    PermanentRedirect = 308,
    /// 400 Bad Request
    BadRequest = 400,
    /// 401 Unauthorized
    Unauthorized = 401,
    /// 402 Payment Required
    PaymentRequired = 402,
    /// 403 Forbidden
    Forbidden = 403,
    /// 404 Not Found
    NotFound = 404,
    /// 405 Method Not Allowed
    MethodNotAllowed = 405,
    /// 406 Not Acceptable
    NotAcceptable = 406,
    /// 407 Proxy Authentication Required
    ProxyAuthenticationRequired = 407,
    /// 408 Request Timeout
    RequestTimeout = 408,
    /// 409 Conflict
    Conflict = 409,
    /// 410 Gone
    Gone = 410,
    /// 411 Length Required
    LengthRequired = 411,
    /// 412 Precondition Failed
    PreconditionFailed = 412,
    /// 413 Payload Too Large
    PayloadTooLarge = 413,
    /// 414 URI Too Long
    URITooLong = 414,
    /// 415 Unsupported Media Type
    UnsupportedMediaType = 415,
    /// 416 Range Not Satisfiable
    RangeNotSatisfiable = 416,
    /// 417 Expectation Failed
    ExpectationFailed = 417,
    /// 418 I'm a teapot
    ImATeapot = 418,
    /// 421 Misdirected Request
    MisdirectedRequest = 421,
    /// 422 Unprocessable Entity
    UnprocessableEntity = 422,
    /// 423 Locked
    Locked = 423,
    /// 424 Failed Dependency
    FailedDependency = 424,
    /// 425 Too Early
    TooEarly = 425,
    /// 426 Upgrade Required
    UpgradeRequired = 426,
    /// 428 Precondition Required
    PreconditionRequired = 428,
    /// 429 Too Many Requests
    TooManyRequests = 429,
    /// 431 Request Header Fields Too Large
    RequestHeaderFieldsTooLarge = 431,
    /// 451 Unavailable For Legal Reasons
    UnavailableForLegalReasons = 451,
    /// 500 Internal Server Error
    InternalServerError = 500,
    /// 501 Not Implemented
    NotImplemented = 501,
    /// 502 Bad Gateway
    BadGateway = 502,
    /// 503 Service Unavailable
    ServiceUnavailable = 503,
    /// 504 Gateway Timeout
    GatewayTimeout = 504,
    /// 505 HTTP Version Not Supported
    HTTPVersionNotSupported = 505,
    /// 506 Variant Also Negotiates
    VariantAlsoNegotiates = 506,
    /// 507 Insufficient Storage
    InsufficientStorage = 507,
    /// 508 Loop Detected
    LoopDetected = 508,
    /// 510 Not Extended
    NotExtended = 510,
    /// 511 Network Authentication Required
    NetworkAuthenticationRequired = 511,
}

impl TryFrom<i32> for Status {
    type Error = InvalidStatusError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            100 => Ok(Status::Continue),
            101 => Ok(Status::SwitchingProtocols),
            102 => Ok(Status::Processing),
            200 => Ok(Status::Ok),
            201 => Ok(Status::Created),
            202 => Ok(Status::Accepted),
            203 => Ok(Status::NonAuthoritativeInformation),
            204 => Ok(Status::NoContent),
            205 => Ok(Status::ResetContent),
            206 => Ok(Status::PartialContent),
            207 => Ok(Status::MultiStatus),
            208 => Ok(Status::AlreadyReported),
            226 => Ok(Status::IMUsed),
            300 => Ok(Status::MultipleChoices),
            301 => Ok(Status::MovedPermanently),
            302 => Ok(Status::Found),
            303 => Ok(Status::SeeOther),
            304 => Ok(Status::NotModified),
            305 => Ok(Status::UseProxy),
            307 => Ok(Status::TemporaryRedirect),
            308 => Ok(Status::PermanentRedirect),
            400 => Ok(Status::BadRequest),
            401 => Ok(Status::Unauthorized),
            402 => Ok(Status::PaymentRequired),
            403 => Ok(Status::Forbidden),
            404 => Ok(Status::NotFound),
            405 => Ok(Status::MethodNotAllowed),
            406 => Ok(Status::NotAcceptable),
            407 => Ok(Status::ProxyAuthenticationRequired),
            408 => Ok(Status::RequestTimeout),
            409 => Ok(Status::Conflict),
            410 => Ok(Status::Gone),
            411 => Ok(Status::LengthRequired),
            412 => Ok(Status::PreconditionFailed),
            413 => Ok(Status::PayloadTooLarge),
            414 => Ok(Status::URITooLong),
            415 => Ok(Status::UnsupportedMediaType),
            416 => Ok(Status::RangeNotSatisfiable),
            417 => Ok(Status::ExpectationFailed),
            418 => Ok(Status::ImATeapot),
            421 => Ok(Status::MisdirectedRequest),
            422 => Ok(Status::UnprocessableEntity),
            423 => Ok(Status::Locked),
            424 => Ok(Status::FailedDependency),
            425 => Ok(Status::TooEarly),
            426 => Ok(Status::UpgradeRequired),
            428 => Ok(Status::PreconditionRequired),
            429 => Ok(Status::TooManyRequests),
            431 => Ok(Status::RequestHeaderFieldsTooLarge),
            451 => Ok(Status::UnavailableForLegalReasons),
            500 => Ok(Status::InternalServerError),
            501 => Ok(Status::NotImplemented),
            502 => Ok(Status::BadGateway),
            503 => Ok(Status::ServiceUnavailable),
            504 => Ok(Status::GatewayTimeout),
            505 => Ok(Status::HTTPVersionNotSupported),
            506 => Ok(Status::VariantAlsoNegotiates),
            507 => Ok(Status::InsufficientStorage),
            508 => Ok(Status::LoopDetected),
            510 => Ok(Status::NotExtended),
            511 => Ok(Status::NetworkAuthenticationRequired),
            _ => Err(InvalidStatusError),
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Status::Continue => "100 Continue",
                Status::SwitchingProtocols => "101 Switching Protocols",
                Status::Processing => "102 Processing",
                Status::Ok => "200 OK",
                Status::Created => "201 Created",
                Status::Accepted => "202 Accepted",
                Status::NonAuthoritativeInformation => "203 Non-Authoritative Information",
                Status::NoContent => "204 No Content",
                Status::ResetContent => "205 Reset Content",
                Status::PartialContent => "206 Partial Content",
                Status::MultiStatus => "207 Multi-Status",
                Status::AlreadyReported => "208 Already Reported",
                Status::IMUsed => "226 IM Used",
                Status::MultipleChoices => "300 Multiple Choices",
                Status::MovedPermanently => "301 Moved Permanently",
                Status::Found => "302 Found",
                Status::SeeOther => "303 See Other",
                Status::NotModified => "304 Not Modified",
                Status::UseProxy => "305 Use Proxy",
                Status::TemporaryRedirect => "307 Temporary Redirect",
                Status::PermanentRedirect => "308 Permanent Redirect",
                Status::BadRequest => "400 Bad Request",
                Status::Unauthorized => "401 Unauthorized",
                Status::PaymentRequired => "402 Payment Required",
                Status::Forbidden => "403 Forbidden",
                Status::NotFound => "404 Not Found",
                Status::MethodNotAllowed => "405 Method Not Allowed",
                Status::NotAcceptable => "406 Not Acceptable",
                Status::ProxyAuthenticationRequired => "407 Proxy Authentication Required",
                Status::RequestTimeout => "408 Request Timeout",
                Status::Conflict => "409 Conflict",
                Status::Gone => "410 Gone",
                Status::LengthRequired => "411 Length Required",
                Status::PreconditionFailed => "412 Precondition Failed",
                Status::PayloadTooLarge => "413 Payload Too Large",
                Status::URITooLong => "414 URI Too Long",
                Status::UnsupportedMediaType => "415 Unsupported Media Type",
                Status::RangeNotSatisfiable => "416 Range Not Satisfiable",
                Status::ExpectationFailed => "417 Expectation Failed",
                Status::ImATeapot => "418 I'm a teapot",
                Status::MisdirectedRequest => "421 Misdirected Request",
                Status::UnprocessableEntity => "422 Unprocessable Entity",
                Status::Locked => "423 Locked",
                Status::FailedDependency => "424 Failed Dependency",
                Status::TooEarly => "425 Too Early",
                Status::UpgradeRequired => "426 Upgrade Required",
                Status::PreconditionRequired => "428 Precondition Required",
                Status::TooManyRequests => "429 Too Many Requests",
                Status::RequestHeaderFieldsTooLarge => "431 Request Header Fields Too Large",
                Status::UnavailableForLegalReasons => "451 Unavailable For Legal Reasons",
                Status::InternalServerError => "500 Internal Server Error",
                Status::NotImplemented => "501 Not Implemented",
                Status::BadGateway => "502 Bad Gateway",
                Status::ServiceUnavailable => "503 Service Unavailable",
                Status::GatewayTimeout => "504 Gateway Timeout",
                Status::HTTPVersionNotSupported => "505 HTTP Version Not Supported",
                Status::VariantAlsoNegotiates => "506 Variant Also Negotiates",
                Status::InsufficientStorage => "507 Insufficient Storage",
                Status::LoopDetected => "508 Loop Detected",
                Status::NotExtended => "510 Not Extended",
                Status::NetworkAuthenticationRequired => "511 Network Authentication Required",
            }
        )
    }
}

// MARK: InvalidStatusError
#[derive(Debug)]
pub struct InvalidStatusError;

impl Display for InvalidStatusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid HTTP status")
    }
}

impl Error for InvalidStatusError {}
