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

use wasm_bindgen::prelude::*;
use web_sys::{Event, window};

pub struct EventListener {
    target: web_sys::EventTarget,
    name: &'static str,
    callback: Closure<dyn FnMut(Event)>,
}
impl EventListener {
    pub fn new<F>(target: web_sys::EventTarget, name: &'static str, callback: F) -> Self
    where
        F: FnMut(Event) + 'static,
    {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(Event)>);
        target
            .add_event_listener_with_callback(name, callback.as_ref().unchecked_ref())
            .unwrap();

        Self {
            target,
            name,
            callback,
        }
    }
}
impl Drop for EventListener {
    fn drop(&mut self) {
        self.target
            .remove_event_listener_with_callback(self.name, self.callback.as_ref().unchecked_ref())
            .unwrap();
    }
}

pub struct Interval {
    _callback: Closure<dyn FnMut()>,
    handle: i32,
}
impl Interval {
    pub fn new<F>(callback: F, timeout: i32) -> Self
    where
        F: FnMut() + 'static,
    {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut()>);
        let handle = window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                timeout,
            )
            .unwrap();

        Self {
            _callback: callback,
            handle,
        }
    }
}
impl Drop for Interval {
    fn drop(&mut self) {
        window().unwrap().clear_interval_with_handle(self.handle);
    }
}
