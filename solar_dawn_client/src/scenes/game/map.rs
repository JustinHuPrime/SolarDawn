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

use dioxus::{
    core::needs_update,
    html::{geometry::WheelDelta, input_data::MouseButton},
    prelude::*,
};
use solar_dawn_common::{
    CartesianVec2, GameState, Vec2, celestial::Celestial, order::Order, stack::Stack,
};
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, window};

use crate::{
    ClientState,
    scenes::game::{ClientGameSettings, ClientViewSettings},
};

// Major (left-right) radius of a hex
const HEX_SCALE: f32 = 60.0;

const MIN_ZOOM: i32 = -40;
const MAX_ZOOM: i32 = 10;

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
        let transformed_x = (center.x + self.x_offset) * self.zoom() + (width as f32 / 2.0);
        let transformed_y = (center.y + self.y_offset) * self.zoom() + (height as f32 / 2.0);

        // Check if the hex center is within HEX_SCALE (scaled by zoom) of the visible area
        let margin = HEX_SCALE * self.zoom();

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
    auto_orders: ReadSignal<Vec<(Order, bool)>>,
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

        let client_view_settings = &*client_view_settings.read();

        // Apply transformations: translate to center, apply offsets, then zoom
        ctx.translate(width as f64 / 2.0, height as f64 / 2.0)
            .unwrap();
        ctx.scale(
            client_view_settings.zoom() as f64,
            client_view_settings.zoom() as f64,
        )
        .unwrap();
        ctx.translate(
            client_view_settings.x_offset as f64,
            client_view_settings.y_offset as f64,
        )
        .unwrap();

        // Draw hex grid (if not zoomed too far out)
        if client_view_settings.zoom_level >= -20 {
            // Find hex at center of screen (after inverse transform)
            // Center in canvas space is (width/2, height/2), which maps to (0, 0) in world space after our transforms
            let center_hex = CartesianVec2 {
                x: -client_view_settings.x_offset / HEX_SCALE,
                y: -client_view_settings.y_offset / HEX_SCALE,
            }
            .to_axial();

            // Find hex at top-left of screen (after inverse transform)
            // Top-left in canvas space is (0, 0)
            // After transform: (0, 0) -> (-width/2, -height/2) in transformed space
            // Then apply inverse transforms: divide by zoom, subtract offsets
            let top_left_hex = CartesianVec2 {
                x: (-client_view_settings.x_offset
                    - (width as f32 / 2.0) / client_view_settings.zoom())
                    / HEX_SCALE,
                y: (-client_view_settings.y_offset
                    - (height as f32 / 2.0) / client_view_settings.zoom())
                    / HEX_SCALE,
            }
            .to_axial();

            // Calculate distance between center and top-left hex, add 1 for safety margin
            let render_distance = (center_hex - top_left_hex).norm() + 1;
            ctx.save();
            ctx.set_stroke_style_str("#666666");
            ctx.set_line_width(1.0);
            for q in -render_distance..=render_distance {
                for r in i32::max(-render_distance, -q - render_distance)
                    ..=i32::min(render_distance, -q + render_distance)
                {
                    let hex = Vec2 { q, r } + center_hex;
                    if client_view_settings.maybe_visible(hex, width, height) {
                        draw_hex(&ctx, hex);
                    }
                }
            }
            ctx.restore();
        }

        // draw celestials
        let game_state = &*game_state.read();

        for celestial in game_state.celestials.all().values() {
            if client_view_settings.maybe_visible(celestial.position, width, height) {
                draw_celestial(&ctx, celestial, client_view_settings);
                if celestial.orbit_gravity {
                    draw_gravity_arrows(&ctx, celestial.position);
                }
            }
        }

        // draw orders
        let orders = &*orders.read();
        let auto_orders = &*auto_orders.read();

        for order in auto_orders
            .iter()
            .map(|(order, _)| order)
            .chain(orders.iter())
        {
            draw_order(&ctx, order, game_state);
        }

        let client_game_settings = &*client_game_settings.read();

        for stack in game_state.stacks.values() {
            draw_stack(&ctx, stack, client_view_settings, client_game_settings);
        }

        ctx.restore();
    });

    let mut dragging = use_signal(|| Option::<(f64, f64)>::None);

    rsx! {
        canvas {
            id: "canvas",
            class: "w-100 h-100",
            onresize: move |_| {
                needs_update();
            },
            oncontextmenu: move |event| {
                event.prevent_default();
            },
            onmousedown: move |event| {
                dragging
                    .set(
                        Some((event.coordinates().element().x, event.coordinates().element().y)),
                    );
            },
            onmouseup: move |_| {
                dragging.set(None);
            },
            onmousemove: move |event| {
                // deal with cases where the mouse leaves while held and re-enters not held
                if !event.held_buttons().contains(MouseButton::Primary) {
                    dragging.set(None);
                    return;
                }

                let mut dragging = dragging.write();
                if let Some((start_x, start_y)) = *dragging {
                    let mut client_view_settings = client_view_settings.write();
                    let x = event.coordinates().element().x;
                    let y = event.coordinates().element().y;
                    client_view_settings.x_offset
                        += (x - start_x) as f32 / client_view_settings.zoom();
                    client_view_settings.y_offset
                        += (y - start_y) as f32 / client_view_settings.zoom();
                    *dragging = Some((x, y));
                }
            },
            onwheel: move |event| {
                let is_down = match event.delta() {
                    WheelDelta::Pixels(delta) => delta.y > 0.0,
                    WheelDelta::Lines(delta) => delta.y > 0.0,
                    WheelDelta::Pages(delta) => delta.y > 0.0,
                };
                let mut client_view_settings = client_view_settings.write();
                let mouse_x = event.coordinates().element().x as f32;
                let mouse_y = event.coordinates().element().y as f32;

                // Get canvas dimensions
                let canvas = window()
                    .unwrap()
                    .document()
                    .unwrap()
                    .get_element_by_id("canvas")
                    .unwrap()
                    .unchecked_into::<HtmlCanvasElement>();
                let canvas_width = canvas.client_width() as f32;
                let canvas_height = canvas.client_height() as f32;

                // Update zoom level

                // Calculate world position under mouse after zoom
                let old_zoom = client_view_settings.zoom();
                let world_x_before = (mouse_x - canvas_width / 2.0) / old_zoom
                    - client_view_settings.x_offset;

                // Adjust view offset to keep the world position under the mouse the same
                let world_y_before = (mouse_y - canvas_height / 2.0) / old_zoom
                    - client_view_settings.y_offset;
                if is_down {
                    client_view_settings.zoom_level = i32::max(
                        client_view_settings.zoom_level - 1,
                        MIN_ZOOM,
                    );
                } else {
                    client_view_settings.zoom_level = i32::min(
                        client_view_settings.zoom_level + 1,
                        MAX_ZOOM,
                    );
                }
                let new_zoom = client_view_settings.zoom();
                let world_x_after = (mouse_x - canvas_width / 2.0) / new_zoom
                    - client_view_settings.x_offset;
                let world_y_after = (mouse_y - canvas_height / 2.0) / new_zoom
                    - client_view_settings.y_offset;
                client_view_settings.x_offset += world_x_after - world_x_before;
                client_view_settings.y_offset += world_y_after - world_y_before;
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
    ctx.stroke();
}

fn draw_celestial(
    ctx: &CanvasRenderingContext2d,
    celestial: &Celestial,
    client_view_settings: &ClientViewSettings,
) {
    // Convert hex coordinates to cartesian and scale
    let center = celestial.position.cartesian() * HEX_SCALE;

    // Draw circle with radius scaled by HEX_SCALE
    let radius = HEX_SCALE * celestial.radius;

    ctx.save();
    ctx.begin_path();
    ctx.arc(
        center.x as f64,
        center.y as f64,
        radius as f64,
        0.0,
        2.0 * std::f64::consts::PI,
    )
    .unwrap();
    ctx.set_fill_style_str(&celestial.colour);
    ctx.fill();

    // maybe label the celestial
    if client_view_settings.zoom_level >= -10 {
        ctx.set_font("12px \"Science Gothic\"");
        ctx.set_fill_style_str("#000000");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");

        let text_y = if celestial.radius >= 0.5 {
            // Draw the celestial's name in black text centered on the celestial
            center.y as f64
        } else {
            // Draw the celestial's name in black text centered below the celestial
            (center.y + HEX_SCALE * (0.2 + celestial.radius)) as f64
        };

        ctx.fill_text(&celestial.name, center.x as f64, text_y)
            .unwrap();
    }
    ctx.restore();
}

fn draw_gravity_arrows(ctx: &CanvasRenderingContext2d, celestial_position: Vec2<i32>) {
    let celestial_center = celestial_position.cartesian() * HEX_SCALE;

    // Draw arrows in all 6 neighboring hexes pointing toward the celestial
    ctx.save();
    ctx.set_fill_style_str("#000000");
    for neighbor in celestial_position.neighbours() {
        let neighbor_center = neighbor.cartesian() * HEX_SCALE;

        // Calculate direction from neighbor to celestial
        let dx = celestial_center.x - neighbor_center.x;
        let dy = celestial_center.y - neighbor_center.y;
        let length = (dx * dx + dy * dy).sqrt();

        // Normalize direction
        let norm_dx = dx / length;
        let norm_dy = dy / length;

        // Arrow parameters
        let arrow_length = HEX_SCALE * 1.0;
        let shaft_width = HEX_SCALE * 0.2;
        let arrow_head_length = HEX_SCALE * 0.4;
        let arrow_head_width = HEX_SCALE * 0.4;

        // Calculate arrow start and end points
        let start_x = neighbor_center.x - (norm_dx * arrow_length / 2.0);
        let start_y = neighbor_center.y - (norm_dy * arrow_length / 2.0);
        let end_x = neighbor_center.x + (norm_dx * arrow_length / 2.0);
        let end_y = neighbor_center.y + (norm_dy * arrow_length / 2.0);

        // Calculate perpendicular vector for arrow width
        let perp_x = -norm_dy;
        let perp_y = norm_dx;

        // Arrow head base position (where shaft meets head)
        let head_base_x = end_x - norm_dx * arrow_head_length;
        let head_base_y = end_y - norm_dy * arrow_head_length;

        // Draw filled block arrow as a single path
        ctx.begin_path();

        // Start at bottom-left of shaft
        ctx.move_to(
            (start_x - perp_x * shaft_width) as f64,
            (start_y - perp_y * shaft_width) as f64,
        );

        // Bottom-left to bottom-right of shaft
        ctx.line_to(
            (start_x + perp_x * shaft_width) as f64,
            (start_y + perp_y * shaft_width) as f64,
        );

        // Bottom-right of shaft to bottom-right of head base
        ctx.line_to(
            (head_base_x + perp_x * shaft_width) as f64,
            (head_base_y + perp_y * shaft_width) as f64,
        );

        // Bottom-right of head base to outer right of arrowhead
        ctx.line_to(
            (head_base_x + perp_x * arrow_head_width) as f64,
            (head_base_y + perp_y * arrow_head_width) as f64,
        );

        // Outer right to tip
        ctx.line_to(end_x as f64, end_y as f64);

        // Tip to outer left of arrowhead
        ctx.line_to(
            (head_base_x - perp_x * arrow_head_width) as f64,
            (head_base_y - perp_y * arrow_head_width) as f64,
        );

        // Outer left of arrowhead to top-left of head base
        ctx.line_to(
            (head_base_x - perp_x * shaft_width) as f64,
            (head_base_y - perp_y * shaft_width) as f64,
        );

        // Top-left of head base back to start (top-left of shaft)
        ctx.close_path();

        ctx.fill();
    }
    ctx.restore();
}

fn draw_order(ctx: &CanvasRenderingContext2d, order: &Order, game_state: &GameState) {
    match order {
        // Hostility
        Order::Shoot { stack, target, .. } | Order::Board { stack, target } => {
            draw_arrow(
                ctx,
                &ArrowOptions {
                    from: game_state.stacks[stack].position,
                    to: game_state.stacks[target].position,
                    stop_short: true,
                    stem_width: 5.0,
                    head_style: ArrowHeadOptions::OpenV,
                    colour: "#ff0000",
                },
            );
        }
        // Misc Logistics
        Order::Repair {
            stack,
            target_stack,
            ..
        } => {
            draw_arrow(
                ctx,
                &ArrowOptions {
                    from: game_state.stacks[stack].position,
                    to: game_state.stacks[target_stack].position,
                    stop_short: true,
                    stem_width: 5.0,
                    head_style: ArrowHeadOptions::OpenV,
                    colour: "#00ff00",
                },
            );
        }
        // Resource transfers
        Order::ModuleTransfer { stack, to, .. } => match to {
            solar_dawn_common::order::ModuleTransferTarget::Existing(stack_id) => {
                draw_arrow(
                    ctx,
                    &ArrowOptions {
                        from: game_state.stacks[stack].position,
                        to: game_state.stacks[stack_id].position,
                        stop_short: true,
                        stem_width: 1.0,
                        head_style: ArrowHeadOptions::TriangleOutline,
                        colour: "#0000ff",
                    },
                );
            }
            solar_dawn_common::order::ModuleTransferTarget::New(_) => {
                outline_hex(ctx, game_state.stacks[stack].position);
            }
        },
        Order::ResourceTransfer { stack, to, .. } => match to {
            solar_dawn_common::order::ResourceTransferTarget::FloatingPool
            | solar_dawn_common::order::ResourceTransferTarget::Jettison
            | solar_dawn_common::order::ResourceTransferTarget::Module(_) => {
                outline_hex(ctx, game_state.stacks[stack].position);
            }
            solar_dawn_common::order::ResourceTransferTarget::Stack(stack_id) => {
                draw_arrow(
                    ctx,
                    &ArrowOptions {
                        from: game_state.stacks[stack].position,
                        to: game_state.stacks[stack_id].position,
                        stop_short: true,
                        stem_width: 1.0,
                        head_style: ArrowHeadOptions::TriangleOutline,
                        colour: "#0000ff",
                    },
                );
            }
        },
        // Generic "something's happening here"
        Order::Isru { stack, .. }
        | Order::Refine { stack, .. }
        | Order::Build { stack, .. }
        | Order::Salvage { stack, .. }
        | Order::Arm { stack, .. } => {
            outline_hex(ctx, game_state.stacks[stack].position);
        }
        // Movement
        Order::Burn { stack, delta_v, .. } => {
            let stack_ref = &game_state.stacks[stack];
            draw_arrow(
                ctx,
                &ArrowOptions {
                    from: stack_ref.position + stack_ref.velocity,
                    to: stack_ref.position + *delta_v + stack_ref.velocity,
                    stop_short: false,
                    stem_width: 10.0,
                    head_style: ArrowHeadOptions::FilledTriangle,
                    colour: "#000000",
                },
            );
        }
        Order::OrbitAdjust {
            stack,
            target_position,
            clockwise,
            ..
        } => {
            let stack = &game_state.stacks[stack];
            let orbited = game_state
                .celestials
                .with_gravity()
                .find(|&celestial| stack.orbiting(celestial))
                .unwrap();
            let orbit_params = orbited.orbit_parameters(*clockwise);
            let (_, target_velocity) = orbit_params
                .into_iter()
                .find(|(pos, _)| pos == target_position)
                .unwrap();
            draw_arrow(
                ctx,
                &ArrowOptions {
                    from: stack.position,
                    to: *target_position,
                    stop_short: false,
                    stem_width: 1.0,
                    head_style: ArrowHeadOptions::None,
                    colour: "#000000",
                },
            );
            draw_arrow(
                ctx,
                &ArrowOptions {
                    from: *target_position,
                    to: *target_position + target_velocity,
                    stop_short: false,
                    stem_width: 10.0,
                    head_style: ArrowHeadOptions::FilledTriangle,
                    colour: "#000000",
                },
            );
        }
        Order::Land { stack, on, .. } => {
            draw_arrow(
                ctx,
                &ArrowOptions {
                    from: game_state.stacks[stack].position,
                    to: game_state.celestials.get(*on).unwrap().position,
                    stop_short: false,
                    stem_width: 10.0,
                    head_style: ArrowHeadOptions::FilledTriangle,
                    colour: "#000000",
                },
            );
        }
        Order::TakeOff {
            stack,
            from,
            destination,
            clockwise,
            ..
        } => {
            let stack = &game_state.stacks[stack];
            let (position, velocity) = game_state
                .celestials
                .get(*from)
                .unwrap()
                .orbit_parameters(*clockwise)
                .into_iter()
                .find(|(position, _)| position == destination)
                .unwrap();
            draw_arrow(
                ctx,
                &ArrowOptions {
                    from: stack.position,
                    to: position,
                    stop_short: false,
                    stem_width: 1.0,
                    head_style: ArrowHeadOptions::None,
                    colour: "#000000",
                },
            );
            draw_arrow(
                ctx,
                &ArrowOptions {
                    from: position,
                    to: position + velocity,
                    stop_short: false,
                    stem_width: 10.0,
                    head_style: ArrowHeadOptions::FilledTriangle,
                    colour: "#000000",
                },
            );
        }
        // Misc
        Order::NameStack { .. } => {
            // pass
        }
    }
}

/// Options for drawing an arrow
struct ArrowOptions {
    /// Starting space - always starts at the center of this space
    from: Vec2<i32>,
    /// Ending space - either ends here or stops a hex radius short
    to: Vec2<i32>,
    /// Controls if the arrow ends at the center of the destination or stops short
    stop_short: bool,
    /// How to draw the stem of the arrow
    stem_width: f64,
    /// How to draw the head of the arrow
    head_style: ArrowHeadOptions,
    /// Colour of all parts of the arrow
    colour: &'static str,
}
/// Styles for rendering the head of an arrow.
enum ArrowHeadOptions {
    /// No arrow head; the arrow ends with just the stem.
    None,
    /// An open "V" shape at the end of the arrow (like >).
    OpenV,
    /// An outlined triangle at the end of the arrow.
    TriangleOutline,
    /// A filled triangle at the end of the arrow.
    FilledTriangle,
}

/// Draw an arrow
fn draw_arrow(ctx: &CanvasRenderingContext2d, options: &ArrowOptions) {
    // Convert hex coordinates to cartesian and scale
    let from_center = options.from.cartesian() * HEX_SCALE;
    let to_center = options.to.cartesian() * HEX_SCALE;

    // Calculate direction from 'from' to 'to'
    let dx = to_center.x - from_center.x;
    let dy = to_center.y - from_center.y;
    let length = (dx * dx + dy * dy).sqrt();

    if length == 0.0 {
        return; // Can't draw arrow to same position
    }

    // Normalize direction
    let norm_dx = dx / length;
    let norm_dy = dy / length;

    // Calculate arrow end position
    let arrow_end_x = if options.stop_short {
        to_center.x - norm_dx * HEX_SCALE
    } else {
        to_center.x
    };
    let arrow_end_y = if options.stop_short {
        to_center.y - norm_dy * HEX_SCALE
    } else {
        to_center.y
    };

    // Arrow head parameters
    let arrow_head_length = HEX_SCALE * 0.3;
    let arrow_head_width = HEX_SCALE * 0.3;

    // Calculate arrow head base position
    let head_base_x = arrow_end_x - norm_dx * arrow_head_length;
    let head_base_y = arrow_end_y - norm_dy * arrow_head_length;

    // Calculate perpendicular vector for arrow head
    let perp_x = -norm_dy;
    let perp_y = norm_dx;

    ctx.save();
    ctx.set_stroke_style_str(options.colour);
    ctx.set_fill_style_str(options.colour);
    ctx.set_line_width(options.stem_width);

    // Draw the line (shaft)
    ctx.begin_path();
    ctx.move_to(from_center.x as f64, from_center.y as f64);
    if matches!(
        options.head_style,
        ArrowHeadOptions::OpenV | ArrowHeadOptions::None
    ) {
        // Continue shaft to the tip of the arrow head for OpenV and None styles
        ctx.line_to(arrow_end_x as f64, arrow_end_y as f64);
    } else {
        // Stop shaft at the base of the arrow head for filled/outlined triangle styles
        ctx.line_to(head_base_x as f64, head_base_y as f64);
    }
    ctx.stroke();

    // Draw the arrow head based on style
    match options.head_style {
        ArrowHeadOptions::OpenV => {
            // Draw two lines forming a V
            ctx.begin_path();
            ctx.move_to(
                (head_base_x + perp_x * arrow_head_width) as f64,
                (head_base_y + perp_y * arrow_head_width) as f64,
            );
            ctx.line_to(arrow_end_x as f64, arrow_end_y as f64);
            ctx.line_to(
                (head_base_x - perp_x * arrow_head_width) as f64,
                (head_base_y - perp_y * arrow_head_width) as f64,
            );
            ctx.stroke();
        }
        ArrowHeadOptions::TriangleOutline => {
            // Draw triangle outline
            ctx.begin_path();
            ctx.move_to(arrow_end_x as f64, arrow_end_y as f64);
            ctx.line_to(
                (head_base_x + perp_x * arrow_head_width) as f64,
                (head_base_y + perp_y * arrow_head_width) as f64,
            );
            ctx.line_to(
                (head_base_x - perp_x * arrow_head_width) as f64,
                (head_base_y - perp_y * arrow_head_width) as f64,
            );
            ctx.close_path();
            ctx.stroke();
        }
        ArrowHeadOptions::FilledTriangle => {
            // Draw filled triangle
            ctx.begin_path();
            ctx.move_to(arrow_end_x as f64, arrow_end_y as f64);
            ctx.line_to(
                (head_base_x + perp_x * arrow_head_width) as f64,
                (head_base_y + perp_y * arrow_head_width) as f64,
            );
            ctx.line_to(
                (head_base_x - perp_x * arrow_head_width) as f64,
                (head_base_y - perp_y * arrow_head_width) as f64,
            );
            ctx.close_path();
            ctx.fill();
        }
        ArrowHeadOptions::None => {
            // pass
        }
    }

    ctx.restore();
}

/// Draw the hex again, but with a thick black border
fn outline_hex(ctx: &CanvasRenderingContext2d, hex: Vec2<i32>) {
    ctx.save();
    ctx.set_stroke_style_str("#000000");
    ctx.set_line_width(5.0);
    draw_hex(ctx, hex);
    ctx.restore();
}

fn draw_stack(
    ctx: &CanvasRenderingContext2d,
    stack: &Stack,
    client_view_settings: &ClientViewSettings,
    client_game_settings: &ClientGameSettings,
) {
    // todo!()
}
