# Design Document

High concept: 4X board game in space

A simultaneous-turn-resolution 4X inspired by High Frontier and Triplanetary

## The Board

The board consists of a solar system where the inter-planet distances are to scale, but the size of the bodies themselves and the lunar distances are not. The solar system consists of:

- Sol, which can't be landed on and has extra high gravity
- Mercury, Venus, Mars, Phobos, Deimos, Luna, the Galilean moons of Jupiter (Io, Europa, Ganymede, Callisto), the major moons of Saturn (Titan, Rhea, Iapetus, Dione, Tethys), the major moons of Uranus (Titania, Oberon, Umbriel, Ariel), and the major moon of Neptune, Triton, all of which can be landed on, can be mined for either just ore, just ice, or both, and have standard gravity
- Terra, which can be landed on, can't be mined, but does have standard gravity
- the four gas giants Jupiter, Saturn, Uranus, Neptune, which have standard gravity, can't be landed on, but can be skimmed for fuel directly
- the asteroid belt, hosting mostly ore-bearing bodies
- the Kuiper belt, hosting mainly ice-bearing bodies

## Pieces

Player pieces are grouped into stacks, each of which contains some set of modules.

- Miners produce resources based on where they are, at a rate of 1 tonne per turn; mass 10 tonnes
- Fuel skimmers produce fuel when orbiting a gas giant, at a rate of 1 tonne per turn; mass 10 tonnes
- Cargo holds hold up to 20 tonnes of solids (ore, materials); mass 1 tonne empty
- Tanks hold up to 20 tonnes of liquids (water, fuel); mass 1 tonne empty
- Engines produce 20 kN of thrust; mass 1 tonnes
- Warheads, if they hit another stack, deal damage equal to half the stack's max health; mass 1 tonne (but don't detonate if attached to a habitat); note - does not explode if damaged/destroyed
- Guns have a 50% chance at 1 hex away to deal 1 point of damage, exponential falloff with distance; mass 2 tonnes
- Habitats act as a source of control and can effect repairs; mass 10 tonnes
- Refineries turn water into fuel and ore into materials; mass 20 tonnes
- Factories turn materials into modules or effect repairs; mass 50 tonnes
- Armour plate absorbs damage to a stack first; mass 1 tonne

Modules all have two hit points; a damaged module can't function until repaired (except armour plates still take damage first). A damaged module hit again is destroyed and its contents lost (but the module itself can be salvaged) (and armour plates stop taking damage at this point).

Repairs take one tenth the module's mass in materials

## Economy

There are four resources - water, fuel, ore, and materials.

Water and ore are mined from minor bodies. Refineries turn water into fuel and ore into materials. Factories turn materials into modules.

## Turns

A turn is split into three phases, and in each phase, orders may be given to stacks:

1. Logistics - resource transfers, module transfers, resourcing, repair, construction, etc.; has two subphases
    1. Boarding actions
    2. Everything else
2. Combat - shoot a target with a gun, arm, disarm warheads
3. Movement - burn engines to change velocity

At the end of the logistics phase, stack control updates.
At the end of the movement phase, objects move, and warheads detonate.
