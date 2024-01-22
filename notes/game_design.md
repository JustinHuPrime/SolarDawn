# Game Design

Simultaneous-turn-resolution space 4X featuring logistics, customizable and modular ships, customizable and modular weapons, and a Triplanetary-inspired movement system.

## Turn Structure

1. Economic phase (issue production, cargo transfer, and stack transfer orders)
2. Ordnance actions (ordnance launched)
3. Combat actions (gun combat happens)
4. Movement actions (movement orders issued)

## Ships

Everything that isn't a celestial body is a ship or is ordnance.

A ship may contain:

- fuel tanks - a single tank has 10 points of fuel capacity
- cargo holds - a single cargo hold has 10 points of cargo capacity
- engines - an engine provides one hex worth of delta-v per turn, and consumes one point of fuel
- guns - a gun has a 50% chance to hit a target, and does one damage if it hits
- launch clamps - a launch clamp holds one piece of ordnance, and is reloaded at a factory
- habitat modules - repairs 1 damage per economic phase
- miners - mines 2 points of ice or ore from the asteroid per economic phase, automatically
- factories - may create one module or piece of ordnance from materials
- armour plates - no functionality, but act as additional locations that can be hit

## Cargoes

A cargo hold may hold ordnance, ore, materials, and ice

Ordnance is inert within a cargo hold

Ore is converted 1-to-1 into materials at factories

Ice is converted 2-to-1 into fuel at factories

materials is converted into components and ordnance at factories - factories may make as many things as they have materials

1 point of a mine requires 1 point of materials, and reloading a launch clamp requires 20 points of mine

1 point of torpedo requires 1 point of materials, and reloading a launch clamp requires 40 points of torpedo

1 point of nuke requires 15 points of materials, and reloading a launch clamp requires 40 points of nuke

a fuel tank requires 2 points of materials

a cargo hold requires 1 point of materials

a set of civilian engines requires 4 points of materials

a set of military (overload-capable) engines requires 6 points of materials

a gun requires 4 points of materials

a launch clamp requires 2 points of materials

a habitat module requires 2 points of materials

a miner requires 10 points of materials

a factory requires 20 points of materials

an armour plate requires 2 point of materials

## Worked Example: Transport

20 fuel capacity => 4 materials
100 cargo capacity => 10 materials
engines => 4 materials
habitat module => 2 materials
