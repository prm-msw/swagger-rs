//! Support crate for Swagger codegen.
#![warn(missing_docs, missing_debug_implementations)]
#![deny(unused_extern_crates)]

#[cfg(feature = "serdejson")]
extern crate serde;
#[cfg(feature = "serdejson")]
extern crate serde_json;
#[cfg(feature = "serdejson")]
#[cfg(test)]
#[macro_use]
extern crate serde_derive;
extern crate base64;

#[macro_use]
extern crate hyper;

extern crate futures;
extern crate uuid;

use std::error;
use std::fmt;

/// Module for encoding API properties in base64.
pub mod base64_format;
pub use base64_format::ByteArray;

/// Module for encoding Nullable properties.
pub mod nullable_format;
pub use nullable_format::Nullable;

pub mod auth;
pub use auth::{AuthData, Authorization};

pub mod context;
pub use context::{ContextBuilder, ContextWrapper, EmptyContext, Has, Pop, Push};

/// Module with utilities for creating connectors with hyper.
pub mod connector;
pub use connector::{http_connector, https_connector, https_mutual_connector};

pub mod composites;
pub use composites::{CompositeNewService, CompositeService, GetPath, NotFound};

#[allow(deprecated)]
pub mod add_context;
#[allow(deprecated)]
pub use add_context::{AddContext, AddContextNewService, AddContextService};

pub mod drop_context;
pub use drop_context::DropContext;

pub mod request_parser;
pub use request_parser::RequestParser;

header! {
    /// `X-Span-ID` header, used to track a request through a chain of microservices.
    (XSpanId, "X-Span-ID") => [String]
}

/// Wrapper for a string being used as an X-Span-ID.
#[derive(Debug, Clone, Default)]
pub struct XSpanIdString(pub String);

impl XSpanIdString {
    /// Extract an X-Span-ID from a request header if present, and if not
    /// generate a new one.
    pub fn get_or_generate(req: &hyper::Request) -> Self {
        XSpanIdString(
            req.headers()
                .get::<XSpanId>()
                .map(XSpanId::to_string)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        )
    }
}

impl fmt::Display for XSpanIdString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Very simple error type - just holds a description of the error and an HTTP response code.
/// This is useful for human diagnosis and troubleshooting, and allows client applications to
/// respond appropriately to the HTTP error, such as retrying following a 503 Service Unavailable.
#[derive(Clone, Debug)]
pub struct ApiError(pub String, pub hyper::StatusCode);

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let debug: &fmt::Debug = self;
        debug.fmt(f)
    }
}

impl error::Error for ApiError {
    fn description(&self) -> &str {
        "Failed to produce a valid response."
    }
}

impl<'a> From<&'a str> for ApiError {
    fn from(e: &str) -> Self {
        // Use InternalServerError when none is provided
        ApiError(e.to_string(), hyper::StatusCode::InternalServerError)
    }
}

impl From<String> for ApiError {
    fn from(e: String) -> Self {
        // Use InternalServerError when none is provided
        ApiError(e, hyper::StatusCode::InternalServerError)
    }
}

#[cfg(feature = "serdejson")]
impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        ApiError(format!("Response body did not match the schema: {}", e),
                 hyper::StatusCode::InternalServerError)
    }
}
