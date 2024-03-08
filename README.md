# Two Switch Bypass

A possibly over-engineered and disturbingly affordable system for operating two independent latching buttons in a guitar pedal based around the STM32F030F4Px and [Embassy](https://embassy.dev).

Need help implementing this in your own designs? Want something custom? [Let's chat](mailto:hello@heuristic.industries).

#### Goals

We want two switches, each with:

- "Hold for momentary" operation
- Persistent state between power cycles

...and we want it on a budget (ideally under $1 total), with parts readily available for pick and place in small quantities.

## Implementation

Here's an schematic depicting usage of the firmware in this state.

![schematic](https://github.com/heuristic-industries/two-switch/blob/main/schematic.svg)
