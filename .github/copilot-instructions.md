# Solar Dawn - AI Copilot Instructions

Solar Dawn is a simultaneous-turn-resolution 4X space game.

## Architecture

This is a Cargo workspace with three crates:

- `solar_dawn_client` - Dioxus 0.7 web UI (compiles to WASM)
- `solar_dawn_server` - Axum WebSocket server with authoritative game state
- `solar_dawn_common` - Shared game logic with `client`/`server` feature flags

**Critical**: The server is authoritative. Client applies deltas from server but never validates game rules locally.

## Development Workflows

**Client**: Run with `dx serve` (requires Dioxus CLI: `curl -sSL http://dioxus.dev/install.sh | sh`)
**Server**: Run with `cargo run -p solar_dawn_server` (requires `./public/`, `./cert.pem`, `./key.pem` in working directory)
**Linting**: Use `cargo clippy` (project standard, not `cargo check`)

## Code Conventions

- **License**: All files use AGPL-3.0-or-later with copyright headers
- **Safety**: `#![forbid(unsafe_code)]` in all crates
- **Edition**: Rust 2024
- **Serialization**: CBOR (`serde_cbor`) for all client-server communication
- **Feature flags**: Common crate uses `#[cfg(feature = "server")]` and `#[cfg(feature = "client")]` extensively

## Client-Server Protocol

- **WebSocket** connection with binary CBOR messages
- **Initial state**: Server sends full `GameState` on connect
- **Updates**: Server sends `GameStateDelta` patches when phases advance
- **Orders**: Client sends `Vec<Order>` to server each phase
- **Keep-alive**: Text message "PING" prevents connection timeout

See [solar_dawn_common/src/lib.rs](solar_dawn_common/src/lib.rs) for `GameState`, `GameStateDelta`, and `Phase` enum.
See [solar_dawn_common/src/order.rs](solar_dawn_common/src/order.rs) for all `Order` variants.

## Game State Management

- **Phases**: Three per turn - `Logistics`, `Combat`, `Movement` (see `Phase` enum)
- **Turns**: Server increments turn counter after Movement phase
- **Orders**: Server collects orders from all players, then calls `GameState::next()` which returns a delta
- **Determinism**: Server maintains `rand_pcg::Pcg64` RNG; all randomness happens server-side
- **ID Generation**: Server uses `IdGenerator<T, U>` iterators for `StackId`, `ModuleId`, `CelestialId`
- **Persistence**: Server saves CBOR-serialized `GameServerState` to disk; game scenarios are `GameStateInitializer` functions

## Data Patterns

- Use `Arc<T>` for immutable shared data (e.g., `celestials: Arc<CelestialMap>`, `players: Arc<BTreeMap<...>>`)
- Use `BTreeMap` instead of `HashMap` when deterministic iteration order matters
- `PlayerId`, `StackId`, `ModuleId`, `CelestialId` are newtype wrappers around integers
- Common crate uses `#[cfg_attr(feature = "client", derive(Clone))]` pattern for conditional derives

## Dioxus 0.7 Specifics (Client)

Dioxus 0.7 changed all APIs - `cx`, `Scope`, and `use_state` are removed. See [solar_dawn_client/AGENTS.md](solar_dawn_client/AGENTS.md) for Dioxus conventions.

Key patterns in this project:

- `use_store()` for global state management (see `ClientState` enum in [solar_dawn_client/src/main.rs](solar_dawn_client/src/main.rs))
- `asset!()` macro for assets with `AssetOptions::builder().with_hash_suffix(false)` for favicons/manifests
- Custom `WebsocketClient` wrapper in [solar_dawn_client/src/websocket.rs](solar_dawn_client/src/websocket.rs) implements `Stream` trait

## Game Module System

Player pieces are grouped into **stacks**, each containing modules. All modules have 2 hit points; damaged modules don't function (except armor). See [notes/Design.md](notes/Design.md) for full mechanics.

**Production modules** (mass 10t each):
- Miners: produce 1t ore/water per turn from celestial bodies
- Fuel skimmers: produce 1t fuel per turn when orbiting gas giants
- Refineries (20t): convert water→fuel, ore→materials
- Factories (50t): build modules, effect repairs

**Storage modules** (mass 1t empty):
- Cargo holds: 20t solids (ore, materials)
- Tanks: 20t liquids (water, fuel)

**Military modules**:
- Guns (2t): 50% hit chance at 1 hex, exponential falloff
- Warheads (1t): deal 50% of target max health damage on impact
- Armor plates (1t): absorb damage first

**Other modules**:
- Habitats (10t): control source, enable repairs
- Engines (1t): produce 20 kN thrust

Repairs cost 1/10th module mass in materials.

## General Guidelines

- **Do not implement TODOs** unless explicitly instructed
- **Do not run tests** unless specifically needed
- **Ask for context** if instructions are incomplete
- Use clippy for Rust linting, not just cargo check
