#![feature(conservative_impl_trait)]
#![feature(io)]

//! The backblaze api requires https, so you need to provide a Client with a https connector.
//!
//! Such a client can be created with the api call below:
//!
//! ```rust,ignore
//! extern crate hyper;
//! extern crate hyper_native_tls;
//! use hyper::Client;
//! use hyper::net::HttpsConnector;
//! use hyper_native_tls::NativeTlsClient;
//!
//! let ssl = NativeTlsClient::new().unwrap();
//! let connector = HttpsConnector::new(ssl);
//! let client = Client::with_connector(connector);
//! ```
//!
//! Unfortunately because of the hyper api design, the upload functionality in this library
//! requires the connector instead of the client, and since the client consumes the connector,
//! you'll have to make two of them.

extern crate base64;
extern crate serde;
extern crate serde_json;
extern crate core;

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate hyper;

pub mod raw;

use std::fmt;
use hyper::client::Response;

/// TODO: large files

header! { (B2AuthHeader, "Authorization") => [String] }

/// When the B2 api returns an error, it is a json-object that can be deserialized into this object
#[derive(Deserialize, Debug)]
pub struct B2ErrorMessage {
    code: String,
    message: String,
    status: u32
}

/// An error caused while using any of the B2 apis
#[derive(Debug)]
pub enum B2Error {
    HyperError(hyper::error::Error),
    IOError(std::io::Error),
    JsonError(serde_json::Error),
    B2Error(hyper::status::StatusCode, B2ErrorMessage),
    LibraryError(String)
}
impl From<serde_json::Error> for B2Error {
    fn from(err: serde_json::Error) -> B2Error {
        B2Error::JsonError(err)
    }
}
impl From<hyper::error::Error> for B2Error {
    fn from(err: hyper::error::Error) -> B2Error {
        B2Error::HyperError(err)
    }
}
impl From<hyper::error::ParseError> for B2Error {
    fn from(err: hyper::error::ParseError) -> B2Error {
        B2Error::HyperError(hyper::error::Error::Uri(err))
    }
}
impl From<std::io::Error> for B2Error {
    fn from(err: std::io::Error) -> B2Error {
        B2Error::IOError(err)
    }
}
impl B2Error {
    fn from_response(response: Response) -> B2Error {
        let status = response.status;
        let b2err = serde_json::from_reader(response);
        match b2err {
            Ok(errm) =>
                B2Error::B2Error(status, errm),
            Err(json) => B2Error::from(json)
        }
    }
}
impl fmt::Display for B2Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            B2Error::HyperError(ref he) => he.fmt(f),
            B2Error::IOError(ref ioe) => ioe.fmt(f),
            B2Error::JsonError(ref jsonerr) => jsonerr.fmt(f),
            B2Error::B2Error(_, ref b2err) => write!(f, "{} ({}): {}", b2err.status, b2err.code, b2err.message),
            B2Error::LibraryError(ref msg) => write!(f, "{}", msg)
        }
    }
}
