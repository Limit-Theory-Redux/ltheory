-- System planner kernel, part 2 of 6: the timeline of a leg and its reads. A profile is a chain of
-- samples (time, distance along the leg) and, per interval between two samples, the greatest speed
-- in it. A leg's profile is kept per recursion depth, so a retry on the other side of a body finds
-- its parent's intact. Not reentrant: it keeps module scratch state.
-- Design: doc/script/nav/system-planner.md.

local max, min = math.max, math.min

local Profile = {}

local legProfiles = {}

---A new, empty profile.
function Profile.new()
    return { t = {}, s = {}, vmax = {}, count = 0, drive = false }
end

---The profile of the leg at recursion depth `depth`.
function Profile.forDepth(depth)
    local profile = legProfiles[depth + 1]
    if not profile then
        profile = Profile.new()
        legProfiles[depth + 1] = profile
    end
    return profile
end

---Fills `profile` with a leg of `length` flown at `speed` from time `t0`.
function Profile.flat(profile, t0, length, speed)
    profile.t[1], profile.s[1], profile.vmax[1] = t0, 0, speed
    profile.t[2], profile.s[2] = t0 + length / speed, length
    profile.count, profile.drive, profile.through = 2, false, false
end

---The index of the interval of the profile that holds time `t`, searching on from interval `k`.
---An interval ends where the next begins, so a time on a boundary belongs to the later one.
function Profile.locate(profile, t, k)
    local times, last = profile.t, profile.count - 1
    while k < last and t >= times[k + 1] do k = k + 1 end
    return k
end

---How far along the leg the ship is at time `t`, which lies in interval `k`.
function Profile.distanceAt(profile, t, k)
    local t1, t2 = profile.t[k], profile.t[k + 1]
    local s1, s2 = profile.s[k], profile.s[k + 1]
    if t >= t2 or t2 <= t1 then return s2 end
    return s1 + (s2 - s1) * (t - t1) / (t2 - t1)
end

---The time the ship is `along` from the start of the leg, linear between the samples of the
---profile; the leg's start for a distance of zero or less, its end for one past the leg.
function Profile.timeAtDistance(profile, along)
    local times, dists = profile.t, profile.s
    local k, last = 1, profile.count - 1
    while k < last and dists[k + 1] < along do k = k + 1 end
    local s1, s2 = dists[k], dists[k + 1]
    if s2 <= s1 then return times[k] end
    return times[k] + (times[k + 1] - times[k]) * min(1, max(0, (along - s1) / (s2 - s1)))
end

---The ship's mean speed over the interval of the profile that holds the distance `along` from the
---start of the leg (the first interval for a distance of zero or less, the last one past the leg).
function Profile.speedAtDistance(profile, along)
    local times, dists = profile.t, profile.s
    local k, last = 1, profile.count - 1
    while k < last and dists[k + 1] < along do k = k + 1 end
    local dt = times[k + 1] - times[k]
    if dt <= 0 then return 0 end
    return (dists[k + 1] - dists[k]) / dt
end

return Profile
