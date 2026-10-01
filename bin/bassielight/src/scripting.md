# BassieLight stage

This folder is a [BassieLight](https://github.com/bplaat/crates/tree/master/bin/bassielight) stage. BassieLight
regenerates this file every time it opens the stage, don't edit it.

BassieLight watches this folder while the editor or scripts tab is open: changes to `stage.json` and `scripts/*.lua`
show up right away. On the stage tab the setup is frozen, changes are picked up when leaving it.

## Files

- `stage.json`: the room, fixtures, groups and buttons
- `scripts/<name>.lua`: light animation scripts, the file name is the script name

### stage.json

```json
{
  "room": { "width": 800, "height": 500 },
  "fixtures": [{ "id": 1, "name": "Par 1", "type": "american_dj_p56led", "addr": 1, "x": 200, "y": 100 }],
  "groups": [{ "id": 1, "name": "Front", "fixtures": [1, 2], "hide_outline": false }],
  "buttons": [
    { "id": 1, "label": "", "x": 100, "y": 450, "width": 100, "height": 40, "target": { "type": "script", "name": "chase" } }
  ]
}
```

- Sizes and positions are in centimeters, `x` and `y` of fixtures and buttons are their centers.
- `addr` is the DMX start address, fixtures must fit in the 512 DMX channels. Ids must be unique.
- A button `target` is `{ "type": "fixture", "id": 1 }`, `{ "type": "group", "id": 1 }`,
  `{ "type": "script", "name": "chase" }` or `null`. Pressing a script button starts or stops the script.

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

A fixture has the fields `id`, `name`, `type`, `kind` (`rgb`, `switch`, `strobe` or `movingHead`) and its position
`x` and `y`, with the methods `fixture:set(props)`, `fixture:tween(props, beats)` and `fixture:get(prop)`.

### Props

Props are set with a table like `{ color = "red", intensity = 0.5 }`, a prop a fixture doesn't have is ignored.

| Prop | Value | Kinds |
| --- | --- | --- |
| `color`, `toggle_color` | color | rgb, movingHead |
| `intensity` | 0 to 1 | rgb, movingHead |
| `toggle_tween` | `"direct"`, `"linear"` or `"ease"` | rgb, movingHead |
| `toggle_speed`, `strobe_speed` | beats, or `false` for off | rgb, movingHead |
| `preset` | preset name or number, `false` for the own color | fixtures with presets |
| `preset_speed` | 0 to 1 | fixtures with presets |
| `gobo` | gobo name or number, `false` for open | movingHead |
| `focus` | 0 (big) to 1 (small) | movingHead |
| `movement` | movement name or number, `false` to stand still | movingHead |
| `movement_speed` | 0 to 1 | movingHead |
| `switches` | list of booleans like `{ true, false, true, false }` | switch |
| `flash_on` | boolean | strobe |
| `flash_intensity`, `flash_speed` | 0 to 1 | strobe |

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
