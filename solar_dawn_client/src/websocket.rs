// Copyright 2025 Justin Hu
//
// This file is part of Solar Dawn.
//
// Solar Dawn is free software: you can redistribute it and/or modify it under
// the terms of the GNU Affero General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// Solar Dawn is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License
// for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Solar Dawn. If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    cell::RefCell,
    collections::VecDeque,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, Waker},
};

use dioxus::prelude::*;
use futures::Stream;
use solar_dawn_common::KEEP_ALIVE_PING;
use thiserror::Error;
use wasm_bindgen::JsCast;
use web_sys::{
    CloseEvent, MessageEvent, WebSocket,
    js_sys::{ArrayBuffer, JsString, Uint8Array},
};

use crate::event_listener::{EventListener, Interval};

pub enum Message {
    Text(String),
    Binary(Box<[u8]>),
}

#[derive(Error, Debug)]
pub enum WebsocketError {
    #[error("Connection failed")]
    ConnectionFailed,
    #[error("Invalid URL")]
    InvalidURL,
    #[error("Connection closed: {reason}")]
    ConnectionClosed {
        code: u16,
        reason: String,
        was_clean: bool,
    },
}

#[derive(Clone)]
pub struct WebsocketClient {
    raw_ws: WebSocket,
    waker: Rc<RefCell<Option<Waker>>>,
    _on_open: Rc<EventListener>,
    _on_message: Rc<EventListener>,
    _on_error: Rc<EventListener>,
    _on_close: Rc<EventListener>,
    _keep_alive: Rc<Interval>,
    event_queue: Rc<RefCell<VecDeque<Result<Message, WebsocketError>>>>,
}

impl WebsocketClient {
    pub fn send(&self, message: Message) {
        match message {
            Message::Text(text) => {
                self.raw_ws
                    .send_with_str(&text)
                    .expect("websocket should be connected");
            }
            Message::Binary(data) => {
                self.raw_ws
                    .send_with_u8_array(&data)
                    .expect("websocket should be connected");
            }
        }
    }
}
impl Stream for WebsocketClient {
    type Item = Result<Message, WebsocketError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(event) = self.event_queue.borrow_mut().pop_front() {
            trace!("websocket ready on poll with event");
            Poll::Ready(Some(event))
        } else if self.raw_ws.ready_state() == 3 {
            // out of events and the socket is closed
            // must be exactly 3 - see https://websockets.spec.whatwg.org/#closeWebSocket
            trace!("websocket ready on poll with end of stream");
            Poll::Ready(None)
        } else {
            trace!("websocket not ready on poll");
            self.waker.borrow_mut().replace(cx.waker().clone());
            Poll::Pending
        }
    }
}
impl PartialEq for WebsocketClient {
    fn eq(&self, other: &Self) -> bool {
        self.raw_ws == other.raw_ws
    }
}
impl Eq for WebsocketClient {}

pub struct WebsocketClientBuilder(WebsocketClient);

impl WebsocketClientBuilder {
    pub fn new(url: impl AsRef<str>) -> Result<Self, WebsocketError> {
        let raw_ws = WebSocket::new(url.as_ref()).map_err(|_| WebsocketError::InvalidURL)?;
        raw_ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let event_queue = Rc::new(RefCell::new(VecDeque::new()));
        let waker = Rc::new(RefCell::new(None));

        Ok(Self(WebsocketClient {
            raw_ws: raw_ws.clone(),
            waker: waker.clone(),
            _on_open: Rc::new(EventListener::new(raw_ws.clone().into(), "open", {
                let waker = waker.clone();
                move |_| {
                    trace!("websocket got open event");
                    if let Some(waker) = waker.borrow_mut().take() {
                        trace!("websocket waking on open event");
                        waker.wake();
                    }
                }
            })),
            _on_message: Rc::new(EventListener::new(raw_ws.clone().into(), "message", {
                let event_queue = event_queue.clone();
                let waker = waker.clone();
                move |msg| {
                    trace!("websocket got message event");
                    let msg = msg.unchecked_into::<MessageEvent>();
                    if let Ok(msg) = msg.data().dyn_into::<ArrayBuffer>() {
                        event_queue.borrow_mut().push_back(Ok(Message::Binary(
                            Uint8Array::new(&msg).to_vec().into_boxed_slice(),
                        )));
                    } else if let Ok(msg) = msg.data().dyn_into::<JsString>() {
                        event_queue
                            .borrow_mut()
                            .push_back(Ok(Message::Text(msg.into())));
                    } else {
                        // https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/message_event
                        unreachable!("invalid message data type");
                    }

                    if let Some(waker) = waker.borrow_mut().take() {
                        trace!("websocket waking on message event");
                        waker.wake();
                    }
                }
            })),
            _on_error: Rc::new(EventListener::new(raw_ws.clone().into(), "error", {
                let event_queue = event_queue.clone();
                let waker = waker.clone();
                move |_| {
                    trace!("websocket got error event");
                    event_queue
                        .borrow_mut()
                        .push_back(Err(WebsocketError::ConnectionFailed));

                    if let Some(waker) = waker.borrow_mut().take() {
                        trace!("websocket waking on error event");
                        waker.wake();
                    }
                }
            })),
            _on_close: Rc::new(EventListener::new(raw_ws.clone().into(), "close", {
                let event_queue = event_queue.clone();
                let waker = waker.clone();
                move |event| {
                    trace!("websocket got close event");
                    let event = event.unchecked_into::<CloseEvent>();
                    event_queue
                        .borrow_mut()
                        .push_back(Err(WebsocketError::ConnectionClosed {
                            code: event.code(),
                            reason: event.reason(),
                            was_clean: event.was_clean(),
                        }));

                    if let Some(waker) = waker.borrow_mut().take() {
                        trace!("websocket waking on close event");
                        waker.wake();
                    }
                }
            })),
            _keep_alive: Rc::new(Interval::new(
                {
                    let raw_ws = raw_ws.clone();
                    move || {
                        let _ = raw_ws.send_with_str(KEEP_ALIVE_PING);
                    }
                },
                10_000,
            )),
            event_queue: event_queue.clone(),
        }))
    }
}
impl Future for WebsocketClientBuilder {
    type Output = WebsocketClient;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.0.raw_ws.ready_state() {
            0 => {
                // still connecting
                self.0.waker.borrow_mut().replace(cx.waker().clone());
                Poll::Pending
            }
            1..=3 => {
                // note - 2 and 3 (closing, closed) should result in the returned websocket eventually having a ConnectionClosed error
                self.0.waker.borrow_mut().take();
                Poll::Ready(self.0.clone())
            }
            4.. => {
                // https://developer.mozilla.org/en-US/docs/Web/API/WebSocket/readyState
                unreachable!("invalid ready_state")
            }
        }
    }
}
