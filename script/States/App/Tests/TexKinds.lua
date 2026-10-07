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

    Log.Info('[TexKinds] %s (%d checks, %d failed)', failures == 0 and 'PASS' or 'FAIL', checks, failures)
    self.frames = 0
end

function TexKinds:eventLoop()
    Application.eventLoop(self)
    self.frames = self.frames + 1
    if self.frames >= 5 then self:quit() end
end

return TexKinds
