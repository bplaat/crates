# BassieLight stage

This folder is a [BassieLight](https://github.com/bplaat/crates/tree/master/bin/bassielight) stage. BassieLight
regenerates this file every time it opens the stage, don't edit it.

BassieLight watches this folder while the editor or scripts tab is open: changes to `stage.json` and `scripts/*.lua`
show up when there are no unsaved editor changes. On the stage tab the setup is frozen, changes are picked up when leaving it.

## Files

- `stage.json`: the room, fixtures, groups and buttons
- `scripts/<name>.lua`: light animation scripts, organized in real folders if desired

### stage.json

```json
{
  "room": { "width": 800, "height": 500 },
  "fixtures": [{ "id": 1, "name": "Par 1", "type": "american_dj_p56led", "addr": 1, "x": 200, "y": 100 }],
  "groups": [{ "id": 1, "name": "Front", "fixtures": [1, 2], "hide_outline": false }],
  "buttons": [
    { "id": 1, "label": "", "x": 100, "y": 450, "width": 100, "height": 40, "action": "select",
      "targets": [{ "type": "script", "name": "chase" }] },
    { "id": 2, "label": "Front off", "x": 250, "y": 450, "width": 100, "height": 40, "action": "blackout",
      "targets": [{ "type": "group", "id": 1 }, { "type": "fixture", "id": 3 }] }
  ]
}
```

- Sizes and positions are in centimeters, `x` and `y` of fixtures and buttons are their centers.
- A group with `"max_intensity": 0.3` scales every intensity scripts give its fixtures to at most 0.3, so a script
  setting `intensity = 1` gives 0.3 and `0.5` gives 0.15. `fixture:get("intensity")` returns the scaled value.
- `addr` is the DMX start address, fixtures must fit in the 512 DMX channels. Ids must be unique.
- A button has a list of `targets`, each `{ "type": "fixture", "id": 1 }`, `{ "type": "group", "id": 1 }` or
  `{ "type": "script", "name": "chase" }`. Pressing it starts its scripts, or stops them when they all run. Its
  `action` is what it does with its fixtures and groups: `"select"` selects them all for the controls,
  `"blackout"` turns them off like black mode, or back on when they are all off.

Script IDs are paths relative to `scripts/` without `.lua`, for example `Shows/Party Mix` or
`Effects/Pars/Acid Rain`. Use the full ID for script button targets. Folder and file names can contain ASCII
letters, digits, spaces, underscores and hyphens; each component is at most 64 characters. BassieLight shows
all folders in a static tree, including empty folders. Select a folder to create scripts or subfolders in it.

## Scripts

Scripts are [Lua 5.4](https://www.lua.org/manual/5.4/). A script runs from the top when started and stops when it
returns or errors, loop forever to keep an animation going. All timing is in beats of the shared tempo set with the
BPM button, so every script stays in sync with each other and with the music. Scripts start on the next beat.

```lua
-- Chase red through the front group, one fixture per beat
local front = group("Front"):sorted("x")
while true do
    for _, fixture in ipairs(front) do
        front:set({ color = "black" })
        fixture:set({ color = "red" })
        wait(1)
    end
end
```

A script must call `wait` or `sync` regularly, a script that runs too long without waiting is stopped. Scripts have no
access to files or the system, only the `string`, `table`, `math`, `utf8` and `coroutine` libraries are available.

Reusable layers should only change their own fixture group, including any setup at the start. Avoid `mode()` or
resetting `fixtures()` in a layer. Leave `movement` and `movement_speed` untouched unless movement is part of the
effect, even when a full-room show resets colors and brightness. Combine layers on different groups, for example
pars, heads and tubes. Scripts that control the same fixture property compete for it; they are not automatically
mixed. Put full-room shows in `Shows/` (or prefix their names with `Show -`) and run them on their own.
A show can mix two animated RGB looks and tween between their colors
for a crossfade. Stopping a script, or a script that stops with an error, cancels its pending tweens and puts back
the props it changed, except props another running script or the stage controls changed since. A script that returns
keeps its values, so a short script can set a scene.

### Selecting fixtures

- `fixtures()`: selection of all fixtures, `fixtures("rgb")` only the fixtures of a kind
- `group(name)`: selection of the fixtures in a group
- `fixture(name_or_id)`: one fixture

A selection is a list of fixtures, loop over it with `ipairs` and get its size with `#`. It has these methods:

- `selection:set(props)`: set props of all fixtures
- `selection:tween(props, beats)`: fade props of all fixtures to new values over beats, doesn't wait
- `selection:each(function(fixture, index) end)`: call a function for each fixture
- `selection:filter(function(fixture) return true end)`: new selection of the fixtures the function accepts
- `selection:sorted(field)`: new selection sorted by `"x"`, `"y"`, `"name"` or `"id"`

A fixture has the fields `id`, `name`, `type`, `kind` (`rgb`, `switch`, `strobe`, `movingHead` or `haze`) and its position
`x` and `y`, with the methods `fixture:set(props)`, `fixture:tween(props, beats)` and `fixture:get(prop)`.

### Props

Props are set with a table like `{ color = "red", intensity = 0.5 }`, a prop a fixture doesn't have is ignored.

| Prop | Value | Kinds |
| --- | --- | --- |
| `color`, `toggle_color` | color | rgb, movingHead |
| `intensity` | 0 to 1 | rgb, movingHead |
| `toggle_tween` | `"direct"`, `"linear"` or `"ease"` | rgb, movingHead |
| `toggle_speed` | beats, or `false` for off | rgb, movingHead |
| `strobe_speed` | beats, or `false` for continuous output | rgb, movingHead, strobe |
| `preset` | preset name or number, `false` for the own color | fixtures with presets |
| `preset_speed` | 0 to 1 | fixtures with presets |
| `gobo` | gobo name or number, `false` for open | movingHead |
| `focus` | 0 (big) to 1 (small) | movingHead |
| `movement` | movement name or number, `false` to stand still | movingHead |
| `movement_speed` | 0 to 1 | movingHead |
| `switches` | list of booleans like `{ true, false, true, false }` | switch |
| `switch_on`, `switch_all_press` | boolean; turns on all channels, preserving individual settings | switch |
| `flash_on` | boolean | strobe |
| `flash_intensity`, `flash_speed` | 0 to 1 | strobe |
| `haze_on` | boolean; hazers run in every mode and switch off after 1 minute | haze |
| `haze_volume`, `fan_speed` | 0 to 1 | haze |
| `blackout` | boolean; off like in black mode while keeping the other props, moving heads keep moving and hazers keep running | all |

Colors are a name (`black`, `white`, `red`, `green`, `blue`, `yellow`, `magenta`, `cyan`, `orange`, `purple`,
`pink`), a `"#rrggbb"` string or a number like `0xff8000`. Moving heads use the closest color of their color wheel.
`color`, `toggle_color` and all props from 0 to 1 can be tweened.

### Timing

- `wait(beats)`: wait, `wait()` waits one beat
- `sync(beats)`: wait until the next multiple of beats, for example `sync(4)` waits for the next bar
- `beat()`: current position in beats
- `bpm()`: current tempo

### Other

- `mode(name)`: switch between `"manual"`, `"black"` and `"auto"` mode
- `rgb(r, g, b)`: color from 0 to 255 parts, `hsv(hue, saturation, value)`: color from a hue of 0 to 360 and
  saturation and value from 0 to 1
- `colors`: table with the named colors
- `print(...)`: write to the BassieLight log
