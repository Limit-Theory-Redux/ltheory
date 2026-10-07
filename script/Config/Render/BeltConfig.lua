-- Asteroid belt / ring culling (Rust InstanceField, render/instance_field.rs)
Config.render.belt = {
    -- Worker threads for the per-frame belt cull. 0 or 1 = single-threaded on
    -- the main thread. Output is identical for any value. Fields smaller
    -- than ~20k asteroids always run inline. Env LTHEORY_BELT_WORKERS overrides.
    workers = 0,
}
