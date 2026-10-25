local ffi = require('ffi')
local libphx = require('libphx').lib

-- Hand-written exports (the FFI generator cannot return raw pointers), and the
-- transform struct Lua fills in place (`SceneTransform`, render/gpu/scene_list.rs).
ffi.cdef [[
    typedef struct SceneTransform {
        Matrix   world;
        float    cx, cy, cz;
        float    scale;
        uint32_t index;
        uint32_t _pad[3];
    } SceneTransform;

    SceneTransform* SceneList_AddTransform(SceneList*);
    uint8_t const*  SceneList_Visible(SceneList const*);
    float*          SceneList_Users(SceneList*);
]]

local Vec4fPtr = ffi.typeof('Vec4f*')

-- Per-list Lua state, keyed by the list object: the items whose material has a
-- per-draw callback, so `submit` calls it for the survivors only.
--   count            number of callback items this frame (entries past it are stale)
--   items[k]         item index
--   blends[k]        bucket of the item
--   entities[k]      the entity the item was added for
--   fns[k]           the material's callback
local states = setmetatable({}, { __mode = 'k' })

function onDef_SceneList(t, mt)
    t.Create = function()
        local list = Core.ManagedObject(libphx.SceneList_Create(), libphx.SceneList_Free)
        states[list] = { count = 0, items = {}, blends = {}, entities = {}, fns = {} }
        return list
    end
end

-- A `SceneList` collects what the scene passes draw (doc/engine/render-api-v2.md,
-- 3b): `reset()`, then one `addTransform()` per entity and one `addItem(...)`
-- per mesh, then `submit(pass, blend)` once per scene pass. The Lua side only
-- adds the per-draw callbacks (`MaterialType.perDraw`) between Rust's cull and
-- emit steps, so they run for the surviving items and no C-to-Lua call exists.
function onDef_SceneList_t(t, mt)
    local index = mt.__index

    --- Forget last frame's transforms and items.
    index.reset = function(self)
        libphx.SceneList_Reset(self)
        states[self].count = 0
    end

    --- A new transform to fill in place: `t.world` (the body's to-world matrix,
    --- camera-relative), `t.cx/cy/cz` (cull sphere centre, camera-relative) and
    --- `t.scale` (the body's scale; negative = no bounds, never cull). The
    --- pointer is valid until the next `addTransform` or `reset`.
    index.addTransform = function(self)
        return libphx.SceneList_AddTransform(self)
    end

    --- One mesh of the entity whose transform has index `transform`
    --- (`t.index`). `material` is a `Material` instance; the item is drawn in
    --- the bucket of its blend mode. `entity` is handed to the material's
    --- `perDraw` callback. Not allowed inside an open pass.
    index.addItem = function(self, transform, mesh, material, entity)
        local item = libphx.SceneList_AddItem(self, Renderer, transform, mesh, material.handle)
        local perDraw = material.perDraw
        if perDraw then
            local state = states[self]
            local k = state.count + 1
            state.count = k
            state.items[k] = item
            state.blends[k] = material.blend
            state.entities[k] = entity
            state.fns[k] = perDraw
        end
        return item
    end

    --- Draw the bucket `blend` into the open `pass`: frustum cull (unless
    --- `cull == false`), sort by (pipeline, material, mesh) (the alpha bucket
    --- keeps insertion order), call `perDraw(entity, user)` of each surviving
    --- item that has one (`user` is a `Vec4f[7]`: the item's `drawUser`), then
    --- emit the draws.
    index.submit = function(self, pass, blend, cull)
        if cull == nil then cull = true end
        libphx.SceneList_Prepare(self, Renderer, blend, cull)

        local state = states[self]
        local count = state.count
        if count > 0 then
            local visible = libphx.SceneList_Visible(self)
            local users = ffi.cast(Vec4fPtr, libphx.SceneList_Users(self))
            local items, blends, entities, fns = state.items, state.blends, state.entities, state.fns
            for k = 1, count do
                local item = items[k]
                if blends[k] == blend and visible[item] ~= 0 then
                    fns[k](entities[k], users + item * 7)
                end
            end
        end

        libphx.SceneList_Emit(self, Renderer)
    end
end
