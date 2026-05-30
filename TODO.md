# Project Roadmap

This document tracks the ongoing development of the code contained in lib.rs.
None of these are in any specific order.

## UI and Editor Experience
- [ ] Syntax highlighting (might potentially be evil to implement)
- [ ] Fix FL Studio ignoring inputs
- [X] Align code to top of box
- [ ] Improve text caret visibility
- [X] Fix code editor snapping back to bottom when selecting text
- [X] Fix scrolling to the top of long scripts
- [ ] Clean up UI button layout/overflow
- [X] Fix help/documentation text at the top
- [X] Make UI less "weird" (Simple/Advanced mode should probably be infranced from the script itself)

## Core Engine and Stability
- [ ] Infinite loop defense
- [ ] Panic button
- [ ] Save Lua text state
- [ ] Preset management system
- [ ] Reset/Template button
- [X] Refactor lib.rs into multiple modules
- [ ] "Script not ran/compiled yet" indicator
    - Run button should only be selectable if the script was changed

## DevOps and Maintenance
- [ ] Set up GitHub releases
- [ ] Proper semantic versioning
- [ ] Automated build/install script
- [ ] Create images for README

## Future Features
- [ ] UI bindings (create GUI elements and bind GUI controls to Lua variables)
- [ ] Instrument support (create custom instruments rather than just effects, MIDI Input)