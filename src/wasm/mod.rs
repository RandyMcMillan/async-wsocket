// Copyright (c) 2019-2022 Naja Melan
// Copyright (c) 2023-2024 Yuki Kishimoto
// Distributed under the MIT software license

//! Wasm

#![allow(dead_code)] // TODO: remove this
#![allow(clippy::arc_with_non_send_sync)]

use std::time::Duration;

use async_utility::time;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::StreamExt;
use thiserror::Error;
use url::Url;
use wasm_bindgen_futures::spawn_local;

mod error;
mod event;
mod message;
mod pharos;
mod socket;
mod state;
mod stream;

use self::error::WsError;
use self::event::{CloseEvent, WsEvent};
pub use self::message::WsMessage;
use self::pharos::SharedPharos;
use self::socket::WebSocket;
use self::state::WsState;
use self::stream::WsStream;

pub type Sink = SplitSink<WsStream, WsMessage>;
pub type Stream = SplitStream<WsStream>;

#[derive(Debug, Error)]
pub enum Error {
    /// Ws error
    #[error(transparent)]
    Ws(#[from] WsError),
    /// Timeout
    #[error("timeout")]
    Timeout,
}

pub async fn connect(url: &Url, timeout: Option<Duration>) -> Result<(Sink, Stream), Error> {
    let timeout = timeout.unwrap_or(Duration::from_secs(60));
    let (_ws, stream) = time::timeout(Some(timeout), WebSocket::connect(url))
        .await
        .ok_or(Error::Timeout)??;
    Ok(stream.split())
}

/// Helper function to reduce code bloat
pub(crate) fn notify(pharos: SharedPharos<WsEvent>, evt: WsEvent) {
    let notify = async move {
        pharos
            .notify(evt)
            .await
            .map_err(|e| unreachable!("{:?}", e))
            .unwrap(); // only happens if we closed it.
    };
    spawn_local(notify);
}
