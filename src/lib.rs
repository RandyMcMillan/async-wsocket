// Copyright (c) 2022-2024 Yuki Kishimoto
// Distributed under the MIT software license

//! Async WebSocket

#![forbid(unsafe_code)]
#![warn(clippy::large_futures)]
#![cfg_attr(feature = "default", doc = include_str!("../README.md"))]

#[cfg(all(feature = "socks", not(target_arch = "wasm32")))]
use std::net::SocketAddr;
#[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
use std::path::{Path, PathBuf};
use std::time::Duration;

pub use futures_util;
pub use url::{self, Url};

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
pub mod prelude;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use self::native::{Error, Message as WsMessage};
#[cfg(target_arch = "wasm32")]
pub use self::wasm::{Error, WsMessage};

#[cfg(not(target_arch = "wasm32"))]
pub type Sink = Box<dyn futures_util::Sink<WsMessage, Error = Error> + Send + Unpin>;
#[cfg(not(target_arch = "wasm32"))]
pub type Stream = Box<dyn futures_util::Stream<Item = Result<WsMessage, Error>> + Send + Unpin>;
#[cfg(target_arch = "wasm32")]
pub type Sink = Box<dyn futures_util::Sink<WsMessage, Error = Error> + Unpin>;
#[cfg(target_arch = "wasm32")]
pub type Stream = Box<dyn futures_util::Stream<Item = Result<WsMessage, Error>> + Unpin>;

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConnectionMode {
    /// Direct
    #[default]
    Direct,
    /// Custom proxy
    #[cfg(all(feature = "socks", not(target_arch = "wasm32")))]
    Proxy(SocketAddr),
    /// Embedded tor client
    #[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
    Tor {
        /// Path for cache and state data
        ///
        /// Mandatory for `android` and `ios` targets!
        custom_path: Option<PathBuf>,
    },
}

impl ConnectionMode {
    /// Direct connection
    #[inline]
    pub fn direct() -> Self {
        Self::Direct
    }

    /// Proxy
    #[inline]
    #[cfg(all(feature = "socks", not(target_arch = "wasm32")))]
    pub fn proxy(addr: SocketAddr) -> Self {
        Self::Proxy(addr)
    }

    /// Embedded tor client
    ///
    /// This not work on `android` and/or `ios` targets.
    /// Use [`Connection::tor_with_path`] instead.
    #[inline]
    #[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
    pub fn tor() -> Self {
        Self::Tor { custom_path: None }
    }

    /// Embedded tor client
    ///
    /// Specify a path where to store data
    #[inline]
    #[cfg(all(feature = "tor", not(target_arch = "wasm32")))]
    pub fn tor_with_path<P>(data_path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self::Tor {
            custom_path: Some(data_path.as_ref().to_path_buf()),
        }
    }
}

/// Connect
pub async fn connect(
    url: &Url,
    _mode: &ConnectionMode,
    timeout: Duration,
) -> Result<(Sink, Stream), Error> {
    #[cfg(not(target_arch = "wasm32"))]
    let (tx, rx) = self::native::connect(url, _mode, timeout).await?;

    #[cfg(target_arch = "wasm32")]
    let (tx, rx) = self::wasm::connect(url, timeout).await?;

    Ok((tx, rx))
}
