-- Old event class. Will be removed in ECS refactoring
---@deprecated
local OldEvent = {}

function OldEvent.AddedToParent(parent)
    return {
        type = OldEvent.AddedToParent,
        parent = parent,
    }
end

function OldEvent.Broadcast(event)
    return {
        type = OldEvent.Broadcast,
        event = event,
    }
end

function OldEvent.ChildAdded(child)
    return {
        type = OldEvent.ChildAdded,
        child = child,
    }
end

function OldEvent.ChildRemoved(child)
    return {
        type = OldEvent.ChildRemoved,
        child = child,
    }
end

function OldEvent.Damaged(amount, source)
    return {
        type = OldEvent.Damaged,
        amount = amount,
        source = source,
    }
end

function OldEvent.FiredTurret(turret, projectile, effect)
    return {
        type = OldEvent.FiredTurret,
        turret = turret,
        projectile = projectile,
        effect = effect,
    }
end

function OldEvent.Collision(collision, collidedWith)
    return {
        type = OldEvent.Collision,
        collision = collision,
        collidedWith = collidedWith
    }
end

function OldEvent.Debug(context)
    return {
        type = OldEvent.Debug,
        context = context,
    }
end

function OldEvent.Destroyed()
    return {
        type = OldEvent.Destroyed,
    }
end

function OldEvent.RemovedFromParent(parent)
    return {
        type = OldEvent.RemovedFromParent,
        parent = parent,
    }
end

function OldEvent.Render(mode, eye)
    return {
        type = OldEvent.Render,
        mode = mode,
        eye = eye,
    }
end

function OldEvent.Update(dt)
    return {
        type = OldEvent.Update,
        dt = dt,
    }
end

function OldEvent.UpdatePost(dt)
    return {
        type = OldEvent.UpdatePost,
        dt = dt,
    }
end

return OldEvent
