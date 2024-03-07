# Two Switch Bypass

A possibly over-engineered and disturbingly affordable system for operating two independent latching buttons in a guitar pedal based around the STM32F030F4Px and [Embassy](https://embassy.dev).

#### Goals

We want two switches, each with:

- "Hold for momentary operation
- Persistent state between power cycles

...and we want it on a budget (ideally under $1 total), with parts readily available for pick and place in small quantities.

## Implementation

Here's a schematic depicting usage of the firmware in this state.
There's lots of room to make it do other things, or work with other EEPROMs.

![schematic](https://github.com/heuristic-industries/two-switch/blob/main/schematic.svg)
