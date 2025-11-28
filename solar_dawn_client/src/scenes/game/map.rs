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

use dioxus::{core::needs_update, prelude::*};
use solar_dawn_common::{CartesianVec2, GameState, Vec2, order::Order};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, window};

use crate::{
    ClientState,
    scenes::game::{ClientGameSettings, ClientViewSettings},
};

// Major (left-right) radius of a hex
const HEX_SCALE: f32 = 60.0;

impl ClientViewSettings {
    /// Is this hex possibly visible, given the transforms applied?
    ///
    /// false means that it is never visible
    /// true means that it might be visible, or it might be just out of view
    fn maybe_visible(&self, hex: Vec2<i32>, width: u32, height: u32) -> bool {
        // Convert hex to cartesian coordinates and scale
        let center = hex.cartesian() * HEX_SCALE;

        // Apply the same transformations as the canvas:
        // 1. Apply offset
        // 2. Apply zoom
        // 3. Translate to center of canvas
        let transformed_x = (center.x + self.x_offset) * self.zoom + (width as f32 / 2.0);
        let transformed_y = (center.y + self.y_offset) * self.zoom + (height as f32 / 2.0);

        // Check if the hex center is within HEX_SCALE (scaled by zoom) of the visible area
        let margin = HEX_SCALE * self.zoom;

        transformed_x >= -margin
            && transformed_x <= width as f32 + margin
            && transformed_y >= -margin
            && transformed_y <= height as f32 + margin
    }
}

#[component]
pub fn Map(
    game_state: ReadSignal<GameState>,
    orders: ReadSignal<Vec<Order>>,
    client_game_settings: ReadSignal<ClientGameSettings>,
    client_view_settings: WriteSignal<ClientViewSettings>,
    change_state: EventHandler<ClientState>,
) -> Element {
    use_effect(move || {
        let canvas = window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .unchecked_into::<HtmlCanvasElement>();
        let width = canvas.client_width() as u32;
        let height = canvas.client_height() as u32;
        canvas.set_width(width);
        canvas.set_height(height);

        let Some(ctx) = canvas.get_context("2d").unwrap() else {
            change_state(ClientState::Error(
                "canvas rendering not supported".to_owned(),
            ));
            return;
        };
        let ctx = ctx.unchecked_into::<CanvasRenderingContext2d>();
        ctx.save();

        // Clear canvas
        ctx.clear_rect(0.0, 0.0, width as f64, height as f64);

        let view_settings = client_view_settings.read();

        // Apply transformations: translate to center, apply offsets, then zoom
        ctx.translate(width as f64 / 2.0, height as f64 / 2.0)
            .unwrap();
        ctx.scale(view_settings.zoom as f64, view_settings.zoom as f64)
            .unwrap();
        ctx.translate(view_settings.x_offset as f64, view_settings.y_offset as f64)
            .unwrap();

        // Draw hex grid

        // Find hex at center of screen (after inverse transform)
        // Center in canvas space is (width/2, height/2), which maps to (0, 0) in world space after our transforms
        let center_hex = CartesianVec2 {
            x: -view_settings.x_offset / HEX_SCALE,
            y: -view_settings.y_offset / HEX_SCALE,
        }
        .to_axial();

        // Find hex at top-left of screen (after inverse transform)
        // Top-left in canvas space is (0, 0)
        // After transform: (0, 0) -> (-width/2, -height/2) in transformed space
        // Then apply inverse transforms: divide by zoom, subtract offsets
        let top_left_hex = CartesianVec2 {
            x: (-view_settings.x_offset - (width as f32 / 2.0) / view_settings.zoom) / HEX_SCALE,
            y: (-view_settings.y_offset - (height as f32 / 2.0) / view_settings.zoom) / HEX_SCALE,
        }
        .to_axial();

        // Calculate distance between center and top-left hex, add 1 for safety margin
        let render_distance = (center_hex - top_left_hex).norm() + 1;
        debug!(render_distance = render_distance);
        for q in -render_distance..=render_distance {
            for r in i32::max(-render_distance, -q - render_distance)
                ..=i32::min(render_distance, -q + render_distance)
            {
                let hex = Vec2 { q, r } + center_hex;
                if view_settings.maybe_visible(hex, width, height) {
                    draw_hex(&ctx, hex);
                }
            }
        }

        // TODO: draw celestials

        // TODO: draw orders

        // TODO: draw stacks

        // Restore canvas context
        ctx.restore();

        let _ = game_state.read();
        let _ = orders.read();
        let _ = client_game_settings.read();
    });

    rsx! {
        canvas {
            id: "canvas",
            class: "w-100 h-100",
            onresize: move |_| {
                needs_update();
            },
        }
    }
}

fn draw_hex(ctx: &CanvasRenderingContext2d, hex: Vec2<i32>) {
    // Convert hex coordinates to cartesian and scale
    let center = hex.cartesian() * HEX_SCALE;

    // A flat-top hexagon has 6 vertices
    // Starting from the rightmost point and going counter-clockwise
    ctx.begin_path();

    for i in 0..6 {
        let angle = std::f64::consts::PI / 3.0 * i as f64;
        let x = center.x as f64 + HEX_SCALE as f64 * angle.cos();
        let y = center.y as f64 + HEX_SCALE as f64 * angle.sin();

        if i == 0 {
            ctx.move_to(x, y);
        } else {
            ctx.line_to(x, y);
        }
    }

    ctx.close_path();
    ctx.set_stroke_style_str("#000000");
    ctx.set_line_width(1.0);
    ctx.stroke();
}
