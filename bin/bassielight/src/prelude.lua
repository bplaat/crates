-- Copyright (c) 2026 Bastiaan van der Plaat
--
-- SPDX-License-Identifier: MIT

-- Script API on top of the primitives of the script engine, see scripting.md
local bl = ...

local Fixture = {}
Fixture.__index = Fixture

function Fixture:set(props)
    bl.set({ self.id }, props)
    return self
end

function Fixture:tween(props, beats)
    bl.tween({ self.id }, props, beats)
    return self
end

function Fixture:get(prop)
    return bl.get(self.id, prop)
end

local Selection = {}
Selection.__index = Selection

local function selection(list)
    return setmetatable(list, Selection)
end

local function ids(list)
    local result = {}
    for index, fixture in ipairs(list) do
        result[index] = fixture.id
    end
    return result
end

function Selection:set(props)
    bl.set(ids(self), props)
    return self
end

function Selection:tween(props, beats)
    bl.tween(ids(self), props, beats)
    return self
end

function Selection:each(callback)
    for index, fixture in ipairs(self) do
        callback(fixture, index)
    end
    return self
end

function Selection:filter(accept)
    local result = {}
    for _, fixture in ipairs(self) do
        if accept(fixture) then
            result[#result + 1] = fixture
        end
    end
    return selection(result)
end

function Selection:sorted(field)
    local result = { table.unpack(self) }
    table.sort(result, function(a, b)
        return a[field] < b[field]
    end)
    return selection(result)
end

local function all()
    local result = {}
    for index, data in ipairs(bl.fixtures()) do
        result[index] = setmetatable(data, Fixture)
    end
    return result
end

function fixtures(kind)
    local result = selection(all())
    if kind then
        return result:filter(function(fixture)
            return fixture.kind == kind
        end)
    end
    return result
end

function fixture(name)
    for _, fixture in ipairs(all()) do
        if fixture.name == name or fixture.id == name then
            return fixture
        end
    end
    error("Unknown fixture " .. tostring(name), 2)
end

function group(name)
    local members = bl.group(name)
    if not members then
        error("Unknown group " .. tostring(name), 2)
    end
    local byId = {}
    for _, fixture in ipairs(all()) do
        byId[fixture.id] = fixture
    end
    local result = {}
    for _, id in ipairs(members) do
        result[#result + 1] = byId[id]
    end
    return selection(result)
end

function wait(beats)
    coroutine.yield("wait", beats or 1)
end

function sync(beats)
    coroutine.yield("sync", beats or 1)
end

beat = bl.beat
bpm = bl.bpm
mode = bl.mode
colors = bl.colors

local function clamp(value)
    return math.max(0, math.min(255, math.floor(value + 0.5)))
end

function rgb(r, g, b)
    return clamp(r) << 16 | clamp(g) << 8 | clamp(b)
end

function hsv(hue, saturation, value)
    local h = (hue % 360) / 60
    local c = value * saturation
    local x = c * (1 - math.abs(h % 2 - 1))
    local m = value - c
    local r, g, b
    if h < 1 then r, g, b = c, x, 0
    elseif h < 2 then r, g, b = x, c, 0
    elseif h < 3 then r, g, b = 0, c, x
    elseif h < 4 then r, g, b = 0, x, c
    elseif h < 5 then r, g, b = x, 0, c
    else r, g, b = c, 0, x end
    return rgb((r + m) * 255, (g + m) * 255, (b + m) * 255)
end
