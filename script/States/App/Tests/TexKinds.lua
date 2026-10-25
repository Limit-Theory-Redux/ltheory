--- Texture kinds, mip chains and format-agnostic uploads (render API v2, S9).
--- Creates a 1D, 2D, 3D and cube texture (with `desc` mips), uploads data in
--- layouts that differ from the texture's format, reads it back, and logs one
--- `[TexKinds]` line per check and a PASS/FAIL summary. Run it with
--- `ltr -e ./script/Main.lua TexKinds` (also under `LTHEORY_GL_CHECK=1`).
local Application = require('States.Application')

---@class TexKinds: Application
local TexKinds = Subclass('TexKinds', Application)

function TexKinds:getTitle() return 'Texture kinds' end

local failures, checks = 0, 0

local function check(name, ok, detail)
    checks = checks + 1
    if not ok then failures = failures + 1 end
    Log.Info('[TexKinds] %-28s %s %s', name, ok and 'ok' or 'FAIL', detail or '')
end

--- Read `count` values of `read` ('readU8' / 'readF32') into a table.
local function readAll(bytes, read, count)
    bytes:rewind()
    local out = {}
    for i = 1, count do out[i] = bytes[read](bytes) end
    return out
end

local function sameList(a, b, eps)
    if #a ~= #b then return false end
    for i = 1, #a do
        if math.abs(a[i] - b[i]) > (eps or 0) then return false end
    end
    return true
end

local function writeU8s(values)
    local bytes = Bytes.Create(#values)
    for i = 1, #values do bytes:writeU8(values[i]) end
    return bytes
end

local function writeF32s(values)
    local bytes = Bytes.Create(#values * 4)
    for i = 1, #values do bytes:writeF32(values[i]) end
    return bytes
end

local function summary()
    Log.Info('[TexKinds] %s (%d checks, %d failed)', failures == 0 and 'PASS' or 'FAIL', checks, failures)
end

--- Readbacks through the format-agnostic API (render API v2, S8): `Renderer:readSync` on views (levels, rectangles,
--- cube faces, volume layers) in formats other than the texture's own, and `Renderer:readAsync` tickets that
--- resolve a few frames later (see `TexKinds:eventLoop`).
local pending = {}

local function checkAsync(name, ticket, verify)
    pending[#pending + 1] = { name = name, ticket = ticket, verify = verify }
end

local function readbackChecks()
    -- Levels and sub-rectangles of a 2D RGBA8 texture, and the same data through a ticket.
    do
        local tex = Tex2D.Create(8, 8, TexFormat.RGBA8, { mips = true })
        local px = {}
        for i = 0, 63 do
            px[#px + 1] = i * 4; px[#px + 1] = 255 - i * 4; px[#px + 1] = i; px[#px + 1] = 255
        end
        tex:setDataBytes(writeU8s(px), PixelFormat.RGBA, DataFormat.U8)
        tex:genMipmap()

        local full = readAll(Renderer:readSync(tex:view(), 0, 0, 8, 8, TexFormat.RGBA8), 'readU8', 256)
        check('readSync 2D full', sameList(full, px))

        -- 3 x 2 texels at (2, 3): rows of the image, tightly packed.
        local rect = readAll(Renderer:readSync(tex:view(), 2, 3, 3, 2, TexFormat.RGBA8), 'readU8', 24)
        local expect = {}
        for y = 3, 4 do
            for x = 2, 4 do
                local at = (y * 8 + x) * 4
                for c = 1, 4 do expect[#expect + 1] = px[at + c] end
            end
        end
        check('readSync 2D rect', sameList(rect, expect))

        -- Level 1 is 4 x 4; a box filter averages 2 x 2 texels (GL and wgpu may round differently by one).
        local level1 = Renderer:readSync(tex:mipView(1), 0, 0, 4, 4, TexFormat.RGBA8)
        local l1 = readAll(level1, 'readU8', 64)
        local want = {}
        for c = 0, 3 do
            local sum = 0
            for _, at in ipairs({ 0, 1, 8, 9 }) do sum = sum + px[at * 4 + c + 1] end
            want[c + 1] = sum / 4
        end
        check('readSync 2D mip 1', level1:getSize() == 64 and sameList({ l1[1], l1[2], l1[3], l1[4] }, want, 1.01),
            string.format('size %d first %d,%d,%d,%d', level1:getSize(), l1[1], l1[2], l1[3], l1[4]))

        checkAsync('readAsync 2D full', Renderer:readAsync(tex:view(), 0, 0, 8, 8, TexFormat.RGBA8), function(bytes)
            return sameList(readAll(bytes, 'readU8', 256), px)
        end)
    end

    -- Conversion: RGBA16F storage read as floats and as bytes, R32F read as floats and as clamped bytes.
    do
        local tex = Tex2D.Create(4, 1, TexFormat.RGBA16F)
        local floats = { 0.5, 0.25, 1.0, 1.0,  2.0, 0.0, -1.0, 0.5,  0.125, 0.75, 0.0, 1.0,  1.0, 1.0, 1.0, 0.0 }
        tex:setDataBytes(writeF32s(floats), PixelFormat.RGBA, DataFormat.Float)
        local back = readAll(Renderer:readSync(tex:view(), 0, 0, 4, 1, TexFormat.RGBA32F), 'readF32', 16)
        check('RGBA16F as RGBA32F', sameList(back, floats, 1e-3), table.concat(back, ','))
        local bytes = readAll(Renderer:readSync(tex:view(), 0, 0, 4, 1, TexFormat.RGBA8), 'readU8', 16)
        local expect = {}
        for i, v in ipairs(floats) do expect[i] = math.floor(math.max(0, math.min(1, v)) * 255 + 0.5) end
        check('RGBA16F as RGBA8', sameList(bytes, expect, 1), table.concat(bytes, ','))

        local r32 = Tex2D.Create(2, 2, TexFormat.R32F)
        r32:setDataBytes(writeF32s({ 0.25, 0.5, 2.0, -1.0 }), PixelFormat.Red, DataFormat.Float)
        local asFloat = readAll(Renderer:readSync(r32:view(), 0, 0, 2, 2, TexFormat.R32F), 'readF32', 4)
        check('R32F as R32F', sameList(asFloat, { 0.25, 0.5, 2.0, -1.0 }, 1e-6))
        local asBytes = readAll(Renderer:readSync(r32:view(), 0, 0, 2, 2, TexFormat.RGBA8), 'readU8', 16)
        check('R32F as RGBA8', sameList({ asBytes[1], asBytes[5], asBytes[9], asBytes[13] }, { 64, 128, 255, 0 }, 1)
            and asBytes[4] == 255, table.concat(asBytes, ','))
        checkAsync('readAsync R32F', Renderer:readAsync(r32:view(), 0, 0, 2, 2, TexFormat.R32F), function(bytes)
            return sameList(readAll(bytes, 'readF32', 4), { 0.25, 0.5, 2.0, -1.0 }, 1e-6)
        end)
    end

    -- Cube faces and volume layers through their views.
    do
        local cube = TexCube.Create(4, TexFormat.RGBA8)
        local okAll = true
        for f = 0, 5 do
            local px = {}
            for i = 0, 15 do
                px[#px + 1] = f * 40; px[#px + 1] = i * 16; px[#px + 1] = 255 - i * 16; px[#px + 1] = 255
            end
            cube:setDataBytes(writeU8s(px), CubeFace.Get(f), 0, TexFormat.RGBA8, DataFormat.U8)
        end
        for f = 0, 5 do
            local back = readAll(Renderer:readSync(cube:faceView(CubeFace.Get(f)), 0, 0, 4, 4, TexFormat.RGBA8), 'readU8', 64)
            okAll = okAll and back[1] == f * 40 and back[2] == 0 and back[3] == 255 and back[61] == f * 40
                and back[62] == 240
        end
        check('readSync cube faceView', okAll)
        checkAsync('readAsync cube face 3', Renderer:readAsync(cube:faceView(CubeFace.Get(3)), 0, 0, 4, 4, TexFormat.RGBA8),
            function(bytes)
                local back = readAll(bytes, 'readU8', 64)
                return back[1] == 120 and back[62] == 240
            end)

        local vol = Tex3D.Create(4, 4, 4, TexFormat.R8)
        local px = {}
        for i = 0, 63 do px[#px + 1] = i * 4 end
        vol:setDataBytes(writeU8s(px), PixelFormat.Red, DataFormat.U8)
        local layer = readAll(Renderer:readSync(vol:layerView(2), 0, 0, 4, 4, TexFormat.R8), 'readU8', 16)
        local expect = {}
        for i = 32, 47 do expect[#expect + 1] = i * 4 end
        check('readSync 3D layerView', sameList(layer, expect))

        local lut = Tex1D.Create(8, TexFormat.R8)
        local lutPx = {}
        for i = 0, 7 do lutPx[#lutPx + 1] = i * 30 end
        lut:setDataBytes(writeU8s(lutPx), PixelFormat.Red, DataFormat.U8)
        local part = readAll(Renderer:readSync(lut:view(), 2, 0, 4, 1, TexFormat.R8), 'readU8', 4)
        check('readSync 1D part', sameList(part, { 60, 90, 120, 150 }))
    end
end

function TexKinds:onInit()
    check('RGB8 is gone', TexFormat.RGB8 == nil)
    check('TexUsage bits', TexUsage.Sampled == 1 and TexUsage.CopyDst == 8)

    -- 2D, full chain, RGBA8 bytes in the texture's own layout.
    do
        local tex = Tex2D.Create(8, 8, TexFormat.RGBA8, { mips = true })
        local px = {}
        for i = 0, 63 do
            px[#px + 1] = i * 4; px[#px + 1] = 255 - i * 4; px[#px + 1] = i; px[#px + 1] = 255
        end
        tex:setDataBytes(writeU8s(px), PixelFormat.RGBA, DataFormat.U8)
        tex:genMipmap()
        local back = readAll(tex:getDataBytes(PixelFormat.RGBA, DataFormat.U8), 'readU8', 256)
        check('2D RGBA8 + mips', sameList(back, px), string.format('size %s', tostring(tex:getSize())))
    end

    -- 2D, RG8 (it used to be created as GL_RGB), RG bytes.
    do
        local tex = Tex2D.Create(4, 4, TexFormat.RG8)
        local px = {}
        for i = 0, 15 do px[#px + 1] = i * 16; px[#px + 1] = 255 - i * 16 end
        tex:setDataBytes(writeU8s(px), PixelFormat.RG, DataFormat.U8)
        local back = readAll(tex:getDataBytes(PixelFormat.RG, DataFormat.U8), 'readU8', 32)
        check('2D RG8', sameList(back, px))
    end

    -- 2D, float RGB data into an RGBA8 texture: alpha is 1, floats round to nearest.
    do
        local tex = Tex2D.Create(2, 2, TexFormat.RGBA8)
        local floats = { 0, 0.5, 1,  0.2, 0.4, 0.6,  1, 0, 0,  0, 0, 1 }
        tex:setDataBytes(writeF32s(floats), PixelFormat.RGB, DataFormat.Float)
        local back = readAll(tex:getDataBytes(PixelFormat.RGBA, DataFormat.U8), 'readU8', 16)
        local expect = { 0, 128, 255, 255,  51, 102, 153, 255,  255, 0, 0, 255,  0, 0, 255, 255 }
        check('2D RGB float -> RGBA8', sameList(back, expect, 1), table.concat(back, ','))
    end

    -- 2D, R32F stays a float.
    do
        local tex = Tex2D.Create(2, 2, TexFormat.R32F)
        tex:setDataBytes(writeF32s({ 0.25, 0.5, 2.0, -1.0 }), PixelFormat.Red, DataFormat.Float)
        local back = readAll(tex:getDataBytes(PixelFormat.Red, DataFormat.Float), 'readF32', 4)
        check('2D R32F', sameList(back, { 0.25, 0.5, 2.0, -1.0 }, 1e-6))
    end

    -- 1D with mips, float RGB LUT data (what ColorLUT uploads).
    do
        local tex = Tex1D.Create(8, TexFormat.RGBA8, { mips = true })
        local floats = {}
        for i = 0, 7 do floats[#floats + 1] = i / 7; floats[#floats + 1] = 0.5; floats[#floats + 1] = 1 - i / 7 end
        tex:setDataBytes(writeF32s(floats), PixelFormat.RGB, DataFormat.Float)
        tex:genMipmap()
        local back = readAll(tex:getDataBytes(PixelFormat.RGBA, DataFormat.U8), 'readU8', 32)
        local expect = {}
        for i = 0, 7 do
            expect[#expect + 1] = math.floor(i / 7 * 255 + 0.5); expect[#expect + 1] = 128
            expect[#expect + 1] = math.floor((1 - i / 7) * 255 + 0.5); expect[#expect + 1] = 255
        end
        check('1D float LUT + mips', sameList(back, expect, 1))
    end

    -- 3D with mips.
    do
        local tex = Tex3D.Create(4, 4, 4, TexFormat.R8, { mips = true })
        local px = {}
        for i = 0, 63 do px[#px + 1] = i * 4 end
        tex:setDataBytes(writeU8s(px), PixelFormat.Red, DataFormat.U8)
        tex:genMipmap()
        local back = readAll(tex:getDataBytes(PixelFormat.Red, DataFormat.U8), 'readU8', 64)
        check('3D R8 + mips', sameList(back, px), string.format('size %s', tostring(tex:getSize())))
    end

    -- Cube with mips, one face at a time.
    do
        local tex = TexCube.Create(4, TexFormat.RGBA8, { mips = true })
        local okAll = true
        for f = 0, 5 do
            local px = {}
            for i = 0, 15 do
                px[#px + 1] = f * 40; px[#px + 1] = i * 16; px[#px + 1] = 255 - i * 16; px[#px + 1] = 255
            end
            tex:setDataBytes(writeU8s(px), CubeFace.Get(f), 0, TexFormat.RGBA8, DataFormat.U8)
        end
        tex:genMipmap()
        for f = 0, 5 do
            local back = readAll(tex:getDataBytes(CubeFace.Get(f), 0, TexFormat.RGBA8, DataFormat.U8), 'readU8', 64)
            okAll = okAll and back[1] == f * 40 and back[2] == 0 and back[3] == 255 and back[4] == 255
                and back[61] == f * 40 and back[62] == 240
        end
        check('cube RGBA8 faces + mips', okAll)
    end

    -- The usage bits go through.
    do
        local usage = bit.bor(TexUsage.Sampled, TexUsage.CopyDst, TexUsage.CopySrc)
        local tex = Tex2D.Create(4, 4, TexFormat.RGBA8, { usage = usage })
        check('2D usage bits', tex:getSize().x == 4)
    end

    readbackChecks()
    self.frames = 0
end

function TexKinds:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1

    -- Tickets resolve a few frames after they were issued.
    local waiting = {}
    for _, p in ipairs(pending) do
        if p.ticket:ready() then
            local ok = not p.ticket:failed() and p.verify(p.ticket:data())
            check(p.name, ok, string.format('after %d frames', self.frames))
            p.ticket:free()
        else
            waiting[#waiting + 1] = p
        end
    end
    pending = waiting

    if #pending == 0 and self.frames >= 5 then
        summary()
        self:quit()
    elseif self.frames >= 120 then
        for _, p in ipairs(pending) do check(p.name, false, 'ticket never resolved') end
        summary()
        self:quit()
    end
end

return TexKinds
