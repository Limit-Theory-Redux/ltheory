# Modules

All ECS code is organized in modules located in [Modules](script/Modules) folder.

Each module can contain 3 subfolders (or only some of them): **Entities**, **Components** and **Systems**. Each of those subfolders should contain Lua files with corresponding objects (entities, components and systems) and `__init__.lua` file with declaration of those objects. See [ECS documentation](doc/ecs.md) about how to write ECS code.

`__init__.lua` file example the [components](script/Modules/Physics/Components/__init__.lua) of the Physics module:
```lua
return {
    Mass = require('Modules.Physics.Components.MassComponent'),
    RigidBody = require('Modules.Physics.Components.RigidBodyComponent'),
    Transform = require('Modules.Physics.Components.TransformComponent'),
}
```
