local Bindings = require('States.ApplicationBindings')
local CameraBindings = require('Systems.Controls.Bindings.CameraBindings')
local ShipBindings = require('Systems.Controls.Bindings.ShipBindings')
local Disposition = require('GameObjects.Elements.NPC.Dispositions')
local Entity = require('GameObjects.Entity')
local SocketType = require('GameObjects.Entities.Ship.SocketType')

local HUD = {}
HUD.__index = HUD
setmetatable(HUD, UI.Panel)

HUD.name = 'HUD'
HUD.focusable = true
HUD:setPadUniform(8)

local dockingAllowed = true
local hudFontSize = 14
local lockTimer = 0

local targetsHudPositions = {}
local deltaTime = 0

local updateSensorsInterval = 2 / 10
local deltaSensorsTimer = 0
local barTweak = 0

local updateTargetsInterval = 1 / 60
local lastTargetsUpdate = 0
local deltaTimer = 0

function HUD:drawSystemText(a)
    local cx, cy = self.sx / 2, self.sy / 2

    local hudX = 0
    local hudY = 0
    local hudFsize = hudFontSize
    if GameState.ui.hudStyle == Enums.HudStyles.Wide then
        hudX = 40
        hudY = floor(self.sy / 16)
        hudFsize = hudFontSize + 12
    elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
        hudX = cx - floor(cx / 2)
        hudY = floor(self.sy / 8)
        hudFsize = hudFontSize + 6
    elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
        hudX = cx - 300
        hudY = cy - 280
        hudFsize = hudFontSize
    end

    local text1 = format("System: %s", GameState.world.currentSystem:getName())
    local text2 = format("Location: %s", "XXX")

    -- Draw system name and location
    HUD:drawHudTextDouble(hudX, hudY - 32, Config.ui.color.meterBarLight, hudFsize, 0.0, text1)
    HUD:drawHudTextDouble(hudX, hudY, Config.ui.color.meterBarLight, hudFsize, 0.0, text2)
end

function HUD:drawTargetText(a)
    local player = self.player
    local playerShip = player:getControlling()
    local playerTarget = playerShip:getTarget()

    if playerTarget then
        local cx, cy = self.sx / 2, self.sy / 2

        local subtypetext = ""
        if playerTarget:getType() == Config:getObjectTypeByName("object_types", "Station") then
            subtypetext = Config:getObjectInfo("station_subtypes", playerTarget:getSubType()) .. " "
        end

        local faction
        local factionName = "none"
        if playerTarget.owner then
            faction = playerTarget.owner:getFaction()

            if faction ~= "none" then
                factionName = faction:getName()
            end
        end

        local text1 = format("Target ID: %s", subtypetext .. playerTarget:getName())
        local text2 = format("Target Faction: %s", factionName)

        if playerTarget.usesBoost then
            text1 = text1 .. " [Ace]"
        end
        if playerTarget:isDestroyed() then
            text1 = text1 .. " [destroyed]"
        end

        local longestText = max(#text1, #text2)

        local hudX = 0
        local hudY = 0
        local hudFsize = hudFontSize
        if GameState.ui.hudStyle == Enums.HudStyles.Wide then
            hudX = self.sx - (longestText * 19)
            hudY = floor(self.sy / 16)
            hudFsize = hudFontSize + 12
        elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
            hudX = cx + floor(cx / 4)
            hudY = floor(self.sy / 8)
            hudFsize = hudFontSize + 6
        elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
            hudX = cx + 100
            hudY = cy - 280
            hudFsize = hudFontSize
        end

        -- Draw target faction name and ID
        HUD:drawHudTextDouble(hudX, hudY - 32, Config.ui.color.meterBarLight, hudFsize, 0.0, text1)
        HUD:drawHudTextDouble(hudX, hudY, Config.ui.color.meterBarLight, hudFsize, 0.0, text2)
    end
end

function HUD:drawTargetType(a)
    local player = self.player
    local playerShip = player:getControlling()
    local playerTarget = playerShip:getTarget()

    if playerTarget then
        local cx, cy = self.sx / 2, self.sy / 2

        local hudX = 0
        local hudY = 0
        local hudFsize = hudFontSize
        if GameState.ui.hudStyle == Enums.HudStyles.Wide then
            hudX = cx - 100
            hudY = 120
            hudFsize = hudFontSize
        elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
            hudX = cx - 100
            hudY = cy - floor(cy / 2) - 72
            hudFsize = hudFontSize
        elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
            hudX = cx - 100
            hudY = cy - 224
            hudFsize = hudFontSize
        end

        -- Draw target type
        local text = format("%s", Config:getObjectInfo("object_types", playerTarget:getType()))
        HUD:drawHudTextDouble(hudX, hudY, Config.ui.color.meterBarLight, hudFsize, 0.5, text)
    end
end

function HUD:drawTargetRange(a)
    local player = self.player
    local playerShip = player:getControlling()
    local playerTarget = playerShip:getTarget()

    if playerTarget then
        local cx, cy = self.sx / 2, self.sy / 2

        local hudX = 0
        local hudY = 0
        local hudFsize = hudFontSize
        if GameState.ui.hudStyle == Enums.HudStyles.Wide then
            hudX = cx + 70
            hudY = 120
            hudFsize = hudFontSize
        elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
            hudX = cx + 70
            hudY = floor(cy / 2) - 72
            hudFsize = hudFontSize
        elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
            hudX = cx + 70
            hudY = cy - 224
            hudFsize = hudFontSize
        end

        -- Draw distance from player ship to target
        local text = ""
        if playerShip:getDistance(playerTarget) >= 1000 then
            text = format("Range: %d km", floor(playerShip:getDistance(playerTarget) / 1000 + 0.5))
        else
            text = format("Range: %d m", floor(playerShip:getDistance(playerTarget) + 0.5))
        end

        HUD:drawHudTextDouble(hudX, hudY, Config.ui.color.meterBarLight, hudFsize, 0.5, text)
    end
end

function HUD:drawTargetSubtype(a)
    local player = self.player
    local playerShip = player:getControlling()
    local playerTarget = playerShip:getTarget()

    if playerTarget then
        local targetType = playerTarget:getType()
        local cx, cy = self.sx / 2, self.sy / 2

        local hudX = 0
        local hudY = 0
        local hudFsize = hudFontSize
        if GameState.ui.hudStyle == Enums.HudStyles.Wide then
            hudX = cx - 100
            hudY = 150
            hudFsize = hudFontSize
        elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
            hudX = cx - 100
            hudY = floor(cy / 2) - 48
            hudFsize = hudFontSize
        elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
            hudX = cx - 100
            hudY = cy - 200
            hudFsize = hudFontSize
        end

        if targetType == Config:getObjectTypeByName("object_types", "Ship") or
            targetType == Config:getObjectTypeByName("object_types", "Station") then
            if not playerTarget:isDestroyed() then
                local textSubtype = "Trade" -- default to Station subtype; TODO: use Station hulls/roles
                -- Draw target subtype
                if targetType == Config:getObjectTypeByName("object_types", "Ship") then
                    textSubtype = Config:getObjectInfo("ship_subtypes", playerTarget:getSubType())
                end
                local text = format("%s", textSubtype)
                HUD:drawHudTextDouble(hudX, hudY, Config.ui.color.meterBarLight, hudFsize, 0.5, text)
            end
        elseif targetType == Config:getObjectTypeByName("object_types", "Asteroid") then
            -- Draw asteroid yield type
            if playerTarget:hasYield() then
                local text = playerTarget:getYieldName()
                HUD:drawHudTextDouble(hudX, hudY, Config.ui.color.meterBarLight, hudFsize, 0.5, text)
            end
        end
    end
end

function HUD:drawTargetSpeed(a)
    local player = self.player
    local playerShip = player:getControlling()
    local playerTarget = playerShip:getTarget()

    if playerTarget then
        local targetType = playerTarget:getType()
        local cx, cy = self.sx / 2, self.sy / 2

        local hudX = 0
        local hudY = 0
        local hudFsize = hudFontSize
        if GameState.ui.hudStyle == Enums.HudStyles.Wide then
            hudX = cx + 70
            hudY = 146
            hudFsize = hudFontSize
        elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
            hudX = cx + 70
            hudY = floor(cy / 2) - 48
            hudFsize = hudFontSize
        elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
            hudX = cx + 70
            hudY = cy - 200
            hudFsize = hudFontSize
        end

        if targetType == Config:getObjectTypeByName("object_types", "Ship") or
            targetType == Config:getObjectTypeByName("object_types", "Station") then
            if not playerTarget:isDestroyed() then
                -- Draw target speed
                local text = format("Speed: %s m/s", floor(playerTarget:getSpeed()))
                HUD:drawHudTextDouble(hudX, hudY, Config.ui.color.meterBarLight, hudFsize, 0.5, text)
            end
        elseif targetType == Config:getObjectTypeByName("object_types", "Asteroid") then
            -- Draw asteroid yield size
            if playerTarget:hasYield() then
                local text = format("Yield: %d", playerTarget:getYieldSize())
                HUD:drawHudTextDouble(hudX, hudY, Config.ui.color.meterBarLight, hudFsize, 0.5, text)
            end
        end
    end
end

function HUD:drawTargetMission(a)
    local player = self.player
    local playerShip = player:getControlling()
    local playerTarget = playerShip:getTarget()

    if playerTarget then
        local targetType = playerTarget:getType()
        if targetType == Config:getObjectTypeByName("object_types", "Ship") then
            if not playerTarget:isDestroyed() then
                -- Draw current action (if any) of target name
                if playerTarget:hasActions() then
                    local targetAction = playerTarget:getCurrentAction()
                    if targetAction then
                        local targetActionName = targetAction:getName()
                        local cx, cy = self.sx / 2, self.sy / 2

                        local hudX = 0
                        local hudY = 0
                        local hudFsize = hudFontSize
                        if GameState.ui.hudStyle == Enums.HudStyles.Wide then
                            hudX = cx
                            hudY = 180
                            hudFsize = hudFontSize
                        elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
                            hudX = cx
                            hudY = floor(cy / 2) - 24
                            hudFsize = hudFontSize
                        elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
                            hudX = cx
                            hudY = cy - 170
                            hudFsize = hudFontSize
                        end

                        HUD:drawHudTextDouble(hudX, hudY, Config.ui.color.meterBarLight, hudFsize, 0.5, targetActionName)
                    end
                end
            end
        end
    end
end

function HUD:drawTargetShieldsHullArmor(a)
    -- NOTE: Not used now, but retained temporarily
    local player = self.player
    local playerShip = player:getControlling()
    local playerTarget = playerShip:getTarget()

    if playerTarget then
        local targetType = playerTarget:getType()
        if targetType == Config:getObjectTypeByName("object_types", "Ship") or
            targetType == Config:getObjectTypeByName("object_types", "Station") then
            if not playerTarget:isDestroyed() then
                local cx, cy   = self.sx / 2, self.sy / 2
                local text     = ""

                local hudXs    = 0
                local hudXh    = 0
                local hudXa    = 0
                local hudY     = 0
                local hudFsize = hudFontSize
                if GameState.ui.hudStyle == Enums.HudStyles.Wide then
                    hudXs    = cx - 100
                    hudXh    = cx
                    hudXa    = cx + 100
                    hudY     = 220
                    hudFsize = hudFontSize
                elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
                    hudXs    = cx - 100
                    hudXh    = cx
                    hudXa    = cx + 100
                    hudY     = floor(cy / 2)
                    hudFsize = hudFontSize
                elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
                    hudXs    = cx - 100
                    hudXh    = cx
                    hudXa    = cx + 100
                    hudY     = cy - 130
                    hudFsize = hudFontSize
                end

                -- Draw target shields info
                text = format("Shields")
                HUD:drawHudTextDouble(hudXs, hudY, Config.ui.color.meterBarLight, hudFsize, 0.5, text)
                local targetShieldsPct = floor(playerTarget:mgrShieldGetStrengthPercent() + 0.5)
                text = format("%d%%", targetShieldsPct)
                HUD:drawHudTextDouble(hudXs + 10, hudY + 24, Config.ui.color.meterBarLight, hudFsize, 0.5, text)

                -- Draw target hull info
                text = format("Hull")
                HUD:drawHudTextDouble(hudXh, hudY, Config.ui.color.meterBarLight, hudFsize, 0.5, text)
                local targetHealthPct = floor(playerTarget:mgrHullGetHealthPercent() + 0.5)
                text = format("%d%%", targetHealthPct)
                HUD:drawHudTextDouble(hudXh, hudY + 24, Config.ui.color.meterBarLight, hudFsize, 0.5, text)

                -- Draw target hull armor info
                text = format("Armor")
                HUD:drawHudTextDouble(hudXa, hudY, Config.ui.color.meterBarLight, hudFsize, 0.5, text)
                local targetArmorPct = floor(playerTarget:mgrArmorGetHealthPercent() + 0.5)
                text = format("%d%%", targetArmorPct)
                HUD:drawHudTextDouble(hudXa + 10, hudY + 24, Config.ui.color.meterBarLight, hudFsize, 0.5, text)
            end
        end
    end
end

function HUD:drawPlayerShieldsHullArmor(a)
    -- NOTE: Not used now, but retained temporarily
    local player = self.player
    local playerShip = player:getControlling()

    local cx, cy = self.sx / 2, self.sy / 2
    local text = ""

    local sensorsHeight = 0
    if GameState.ui.sensorsDisplayed then
        sensorsHeight = floor(self.sy / 9)
    end

    local hudXs    = 0
    local hudXh    = 0
    local hudXa    = 0
    local hudY     = 0
    local hudFsize = hudFontSize
    if GameState.ui.hudStyle == Enums.HudStyles.Wide then
        hudXs    = cx - 100
        hudXh    = cx
        hudXa    = cx + 100
        hudY     = self.sy - 160 - sensorsHeight - 74
        hudFsize = hudFontSize
    elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
        hudXs    = cx - 100
        hudXh    = cx
        hudXa    = cx + 100
        hudY     = self.sy - 160 - floor(self.sy / 9) - 74
        hudFsize = hudFontSize
    elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
        hudXs    = cx - 100
        hudXh    = cx
        hudXa    = cx + 100
        hudY     = cy + 26
        hudFsize = hudFontSize
    end

    -- Draw player ship shields info
    text = format("Shields")
    HUD:drawHudTextDouble(hudXs, hudY, Config.ui.color.meterBarLight, hudFontSize, 0.5, text)
    local playerShieldsPct = floor(playerShip:mgrShieldGetStrengthPercent() + 0.5)
    text = format("%d%%", playerShieldsPct)
    HUD:drawHudTextDouble(hudXs + 10, hudY + 24, Config.ui.color.meterBarLight, hudFsize, 0.5, text)

    -- Draw player ship hull info
    text = format("Hull")
    HUD:drawHudTextDouble(hudXh, hudY, Config.ui.color.meterBarLight, hudFontSize, 0.5, text)
    local playerHealthPct = floor(playerShip:mgrHullGetHealthPercent() + 0.5)
    text = format("%d%%", playerHealthPct)
    HUD:drawHudTextDouble(hudXh, hudY + 24, Config.ui.color.meterBarLight, hudFsize, 0.5, text)

    -- Draw player ship hull armor info
    text = format("Armor")
    HUD:drawHudTextDouble(hudXa, hudY, Config.ui.color.meterBarLight, hudFontSize, 0.5, text)
    local playerArmorPct = floor(playerShip:mgrArmorGetHealthPercent() + 0.5)
    text = format("%d%%", playerArmorPct)
    HUD:drawHudTextDouble(hudXa + 10, hudY + 24, Config.ui.color.meterBarLight, hudFsize, 0.5, text)
end

function HUD:drawMissilesLeft(a)
    local player = self.player
    local playerShip = player:getControlling()

    local cx, cy = self.sx / 2, self.sy / 2

    local sensorsHeight = 0
    if GameState.ui.sensorsDisplayed then
        sensorsHeight = floor(self.sy / 9)
    end

    local hudX = 0
    local hudY = 0
    local hudFsize = hudFontSize
    if GameState.ui.hudStyle == Enums.HudStyles.Wide then
        hudX = cx - 150
        hudY = self.sy - 160 - sensorsHeight - 24
        hudFsize = hudFontSize
    elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
        hudX     = cx - 150
        hudY     = self.sy - 160 - floor(self.sy / 9) - 24
        hudFsize = hudFontSize
    elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
        hudX = cx - 150
        hudY = cy + 70
        hudFsize = hudFontSize
    end

    -- Draw player missiles remaining
    local missileCount = 0 -- TODO: get current count of missiles aboard player's ship
    local text = format("Missiles: %d", missileCount)
    HUD:drawHudTextDouble(hudX, hudY, Config.ui.color.meterBarLight, hudFsize, 0.5, text)
end

function HUD:drawPlayerSpeed(a)
    local player = self.player
    local playerShip = player:getControlling()

    local cx, cy = self.sx / 2, self.sy / 2

    local sensorsHeight = 0
    if GameState.ui.sensorsDisplayed then
        sensorsHeight = floor(self.sy / 9)
    end

    local hudX = 0
    local hudY = 0
    local hudFsize = hudFontSize
    if GameState.ui.hudStyle == Enums.HudStyles.Wide then
        hudX = cx
        hudY = self.sy - 160 - sensorsHeight - 24
        hudFsize = hudFontSize
    elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
        hudX     = cx
        hudY     = self.sy - 160 - floor(self.sy / 9) - 24
        hudFsize = hudFontSize
    elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
        hudX = cx
        hudY = cy + 68
        hudFsize = hudFontSize
    end

    -- Draw player ship speed
    local text = format("Speed: %d m/s", floor(playerShip:getSpeed()))
    HUD:drawHudTextDouble(hudX, hudY, Config.ui.color.meterBarBright, hudFsize, 0.5, text)
end

function HUD:drawChaffLeft(a)
    local player = self.player
    local playerShip = player:getControlling()

    local cx, cy = self.sx / 2, self.sy / 2

    local sensorsHeight = 0
    if GameState.ui.sensorsDisplayed then
        sensorsHeight = floor(self.sy / 9)
    end

    local hudX = 0
    local hudY = 0
    local hudFsize = hudFontSize
    if GameState.ui.hudStyle == Enums.HudStyles.Wide then
        hudX = cx + 150
        hudY = self.sy - 160 - sensorsHeight - 24
        hudFsize = hudFontSize
    elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
        hudX     = cx + 150
        hudY     = self.sy - 160 - floor(self.sy / 9) - 24
        hudFsize = hudFontSize
    elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
        hudX = cx + 150
        hudY = cy + 70
        hudFsize = hudFontSize
    end

    -- Draw player chaff remaining
    local chaffCount = 0 -- TODO: get current count of chaff aboard player's ship
    local text = format("Chaff: %d", chaffCount)
    HUD:drawHudTextDouble(hudX, hudY, Config.ui.color.meterBarLight, hudFsize, 0.5, text)
end

function HUD:drawLockWarning(a)
    local player = self.player
    local playerShip = player:getControlling()

    local cx, cy = self.sx / 2, self.sy / 2
    local c = Color(1.0, 0.1, 0.1, a)

    local hudX = 0
    local hudY = 0
    local hudFsize = hudFontSize
    if GameState.ui.hudStyle == Enums.HudStyles.Wide then
        hudX = cx
        hudY = cy + 40
        hudFsize = hudFontSize
    elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
        hudX = cx
        hudY = cy + 40
        hudFsize = hudFontSize
    elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
        hudX = cx
        hudY = cy + 120
        hudFsize = hudFontSize
    end

    -- Draw incoming missile lock on player's ship
    for _, ship in ipairs(GameState.world.currentSystem.ships) do
        -- TEMP: Rather than missile lock, check to see whether player's ship is currently targeted by any other ship
        -- TODO: Change to missile lock only if a missile is locked onto the player's ship
        if ship:getTarget() == playerShip then
            if floor((lockTimer * 10) % 3) == 0 then
                -- TODO: Flash missile lock graphic ~3 times per second
                -- TODO: round those triangle corners!
                UI.DrawEx.Line(hudX, hudY - 10, hudX + 28, hudY + 30, c, false)
                UI.DrawEx.Line(hudX - 28, hudY + 30, hudX + 28, hudY + 30, c, false)
                UI.DrawEx.Line(hudX - 28, hudY + 30, hudX, hudY - 10, c, false)

                UI.DrawEx.Line(hudX, hudY - 2, hudX, hudY + 26, c, true)
                UI.DrawEx.Point(hudX, hudY + 22, 100, c)

                UI.DrawEx.TextAdditive(
                    "UbuntuBold",
                    "LOCK DETECTED",
                    hudFontSize - 5,
                    hudX - 50, hudY + 34, 100, hudFontSize,
                    1.0, 0.2, 0.0, a,
                    0.5, 0.5
                )
            end
        end
    end
end

function HUD:drawWeaponGroups(a)
    local cx, cy = self.sx / 2, self.sy / 2

    local hudMode = 1
    local hudFsize = hudFontSize
    if GameState.ui.hudStyle == Enums.HudStyles.Wide then
        hudMode = 1
        hudFsize = hudFontSize
    elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
        hudMode = 2
        hudFsize = hudFontSize
    elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
        hudMode = 3
        hudFsize = hudFontSize
    end

    local fontsize = 14
    local wgx = { { cx - 280, cx - 240, cx - 200, cx - 160, cx + 160, cx + 200, cx + 240, cx + 280 },
        { cx - 400, cx - 360, cx - 320, cx - 280, cx + 280, cx + 320, cx + 360, cx + 400 },
        { cx - 220, cx - 180, cx - 140, cx - 100, cx + 100, cx + 140, cx + 180, cx + 220 } }
    local wgy = { self.sy - 120,
        self.sy - 160 - floor(self.sy / 9) - 44,
        cy + 96 }

    if GameState.ui.hudStyle == Enums.HudStyles.Wide then
        HUD:drawHudTextDouble(cx - 220, self.sy - 140, Config.ui.color.meterBarLight, hudFontSize, 0.5, "Weapon Groups A")
        HUD:drawHudTextDouble(cx + 220, self.sy - 140, Config.ui.color.meterBarLight, hudFontSize, 0.5, "Weapon Groups B")
    elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
        HUD:drawHudTextDouble(cx - 340, self.sy - 160 - floor(self.sy / 9) - 66, Config.ui.color.meterBarLight,
            hudFontSize,
            0.5, "Weapon Groups A")
        HUD:drawHudTextDouble(cx + 340, self.sy - 160 - floor(self.sy / 9) - 66, Config.ui.color.meterBarLight,
            hudFontSize,
            0.5, "Weapon Groups B")
    end

    for i = 1, 8 do
        if GameState.player.weaponGroup == i then
            UI.DrawEx.Circle(wgx[hudMode][i], wgy[hudMode] + 18, 10, Config.ui.color.meterBarLight, true)
            HUD:drawHudText("UbuntuBold", fontsize + 2, wgx[hudMode][i] - floor(fontsize / 2) - 1, wgy[hudMode] + 10,
                tostring(i), Config.ui.color.borderBright)
        else
            UI.DrawEx.Ring(wgx[hudMode][i], wgy[hudMode] + 18, 10, Config.ui.color.meterBarLight, false)
            HUD:drawHudText("Ubuntu", fontsize, wgx[hudMode][i] - floor(fontsize / 2), wgy[hudMode] + 11, tostring(i),
                Config.ui.color.meterBarLight)
        end
    end
end

function HUD:drawPowerDistro(a)
    local player = self.player
    local playerShip = player:getControlling()

    local cx, cy = self.sx / 2, self.sy / 2

    local hudXLt = 0
    local hudXLm = 0
    local hudXRt = 0
    local hudXRm = 0
    local hudYAt = 0
    local hudYAm = 0
    local hudYBt = 0
    local hudYBm = 0
    local hudFsize = hudFontSize
    if GameState.ui.hudStyle == Enums.HudStyles.Wide then
        hudXLm = cx - 300
        hudXLt = hudXLm - 80
        hudXRm = cx + 140
        hudXRt = hudXRm + 180
        hudYAt = self.sy - 64
        hudYAm = self.sy - 56
        hudYBt = self.sy - 40
        hudYBm = self.sy - 32
        hudFsize = hudFontSize
    elseif GameState.ui.hudStyle == Enums.HudStyles.Balanced then
        hudXLm = cx - 300
        hudXLt = hudXLm - 80
        hudXRm = cx + 140
        hudXRt = hudXRm + 180
        hudYAt = self.sy - 100
        hudYAm = self.sy - 92
        hudYBt = self.sy - 76
        hudYBm = self.sy - 68
        hudFsize = hudFontSize
    elseif GameState.ui.hudStyle == Enums.HudStyles.Tight then
        hudXLm = cx - 240
        hudXLt = hudXLm - 80
        hudXRm = cx + 80
        hudXRt = hudXRm + 180
        hudYAt = cy + 150 - 16
        hudYAm = cy + 150 - 8
        hudYBt = cy + 150 + 6
        hudYBm = cy + 150 + 14
        hudFsize = hudFontSize
    end

    -- Draw player power distribution
    HUD:drawHudTextDouble(hudXLt, hudYAt, Config.ui.color.meterBarLight, hudFsize, 0.0, "Engines")
    HUD:drawHudTextDouble(hudXLt, hudYBt, Config.ui.color.meterBarLight, hudFsize, 0.0, "Sensors")
    HUD:drawHudTextDouble(hudXRt, hudYAt, Config.ui.color.meterBarLight, hudFsize, 0.0, "Weapons")
    HUD:drawHudTextDouble(hudXRt, hudYBt, Config.ui.color.meterBarLight, hudFsize, 0.0, "Shields")

    UI.DrawEx.Meter(hudXLm, hudYAm, 32, 8, Config.ui.color.meterBarLight, 10, 4, 4, true, Config.ui.color.meterBarOver, 1)
    UI.DrawEx.Meter(hudXLm, hudYBm, 32, 8, Config.ui.color.meterBarLight, 10, 4, 1, true, Config.ui.color.meterBarOver, 1)
    UI.DrawEx.Meter(hudXRm, hudYAm, 32, 8, Config.ui.color.meterBarLight, 10, 4, 4, true, Config.ui.color.meterBarOver,
        -1)
    UI.DrawEx.Meter(hudXRm, hudYBm, 32, 8, Config.ui.color.meterBarLight, 10, 4, 3, true, Config.ui.color.meterBarOver,
        -1)
end

function HUD:drawSensors(a)
    if GameState.ui.sensorsDisplayed then
        local cx, cy = self.sx / 2, self.sy / 2

        -- Draw sensor readouts
        local barCount = 48
        local xleft = floor(cx / 2)
        local xlength = floor(cx / barCount) * barCount -- make panel width evenly divisible by number of sensor bars
        local ylength = floor(self.sy / 9)
        local barBase = floor(0.75 * ylength)
        local barReflect = floor(0.25 * ylength)
        local ytop = self.sy - 160 - ylength
        local ybottom = ytop + barBase
        local barWidth = floor(xlength / barCount) - 1

        -- Draw sensor panel background
        UI.DrawEx.Panel(xleft, ytop, xlength, ylength, Config.ui.color.meterBarLight, 0.3)
        UI.DrawEx.Rect(xleft, ybottom, xlength, 4, Config.ui.color.meterBarDark)

        -- Draw sensor bars if game is not paused
        -- TODO: Convert "active" sensor dropoff ranges into passive emission strengths
        if not GameState.paused then
            local maxBarRatio = 0
            local player = self.player
            local playerShip = player:getControlling()
            local playerTarget = playerShip:getTarget()
            if playerTarget then
                -- Draw sensor bars for currently targeted object that isn't destroyed
                if not playerTarget:isDestroyed() then
                    local emType = Enums.Emitters.None
                    local targetType = playerTarget:getType()
                    if targetType == Config:getObjectTypeByName("object_types", "Star") then
                        emType = Enums.Emitters.Star
                    elseif targetType == Config:getObjectTypeByName("object_types", "Planet") then
                        emType = Enums.Emitters.Planet
                    elseif targetType == Config:getObjectTypeByName("object_types", "Station") then
                        emType = Enums.Emitters.Station
                    elseif targetType == Config:getObjectTypeByName("object_types", "Ship") then
                        emType = Enums.Emitters.Ship
                    end

                    if emType ~= Enums.Emitters.None then
                        local rng = RNG.FromTime()
                        local dropoffDist = Config.gen.objectEmissionsDropoff[emType]

                        -- Get the distance from the player's ship to the object
                        local distance = playerShip:getDistance(playerTarget)

                        -- Get a number from 0 - 1 describing how directly the player's ship is looking at an object
                        local align = max(0,
                            (playerTarget:getPos() - playerShip:getPos()):normalize():dot(playerShip:getForward()))

                        -- Calculate signal dropoff as it reaches the player's ship's sensors
                        -- Reduce the strength of the signal itself based on distance (no need to go inverse-square for unnecessary "realism" here)
                        -- Reduce the strength of the received signal based on alignment of the sensor cone with the object
                        -- TODO: Modify strength & accuracy of received signal based on number & quality of ship's Sensor components
                        local dropoff = align * align * (1 - min(dropoffDist, distance) / dropoffDist)

                        local barTweakVals = false
                        deltaSensorsTimer = deltaSensorsTimer + deltaTime
                        if deltaSensorsTimer > updateSensorsInterval then
                            -- Update jitter
                            barTweakVals = true
                            deltaSensorsTimer = 0
                        end

                        for i = 1, barCount do
                            if barTweakVals then
                                -- Add some jitter to the sensor bars
                                barTweak = rng:getUniformRange(-5, 5)
                            end

                            -- Add jitter constrained by the max height of the up and down (reflection) bars
                            local barHeightUp   = min(barBase,
                                floor(Config.gen.objectEmissions[i][emType] * (barBase + barTweak) * dropoff / 100))
                            local barHeightDown = min(barReflect,
                                floor(Config.gen.objectEmissions[i][emType] * (barReflect + barTweak) * dropoff / 100))

                            -- Get the highest bar ratio (greatest frequency bar value divided by the maximum possible bar height)
                            local barRatio      = barHeightUp / barBase
                            if barRatio > maxBarRatio then maxBarRatio = barRatio end

                            -- Finally, actually display all the sensor frequency bars
                            UI.DrawEx.Rect(xleft + ((i - 1) * (barWidth + 1)),
                                ybottom - barHeightUp,
                                barWidth,
                                barHeightUp,
                                Config.ui.color.meterBarBright)
                            UI.DrawEx.Rect(xleft + ((i - 1) * (barWidth + 1)),
                                ybottom,
                                barWidth,
                                barHeightDown,
                                Config.ui.color.meterBarDark)
                        end
                    end
                end
            else
                -- Draw sensor bars for all objects in "cone" projected
                local rng = RNG.FromTime()
                local system = playerShip.parent
                if not playerShip:isShipDocked() and system then -- no displaying Sensor readings while docked at a space station!
                    local stars         = system:getStars()
                    local planets       = system:getPlanets()
                    local stations      = system:getStations()
                    local ships         = system:getShips()
                    local objects       = {}
                    local barHeightUp   = {}
                    local barHeightDown = {}

                    for _, star in ipairs(stars) do
                        if playerShip:getDistance(star) <= Config.gen.objectEmissionsDropoff[Enums.Emitters.Star] then
                            local align = max(0,
                                (star:getPos() - playerShip:getPos()):normalize():dot(playerShip:getForward()))
                            if align * align >= 0.3 then
                                insert(objects, star)
                            end
                        end
                    end
                    for _, planet in ipairs(planets) do
                        if playerShip:getDistance(planet) <= Config.gen.objectEmissionsDropoff[Enums.Emitters.Planet] then
                            local align = max(0,
                                (planet:getPos() - playerShip:getPos()):normalize():dot(playerShip:getForward()))
                            if align * align >= 0.3 then
                                insert(objects, planet)
                            end
                        end
                    end
                    for _, station in ipairs(stations) do
                        if not station:isDestroyed() and playerShip:getDistance(station) <= Config.gen.objectEmissionsDropoff[Enums.Emitters.Station] then
                            local align = max(0,
                                (station:getPos() - playerShip:getPos()):normalize():dot(playerShip:getForward()))
                            if align * align >= 0.3 then
                                insert(objects, station)
                            end
                        end
                    end
                    for _, ship in ipairs(ships) do
                        if ship ~= playerShip and
                            not ship:isDestroyed() and
                            playerShip:getDistance(ship) <= Config.gen.objectEmissionsDropoff[Enums.Emitters.Ship] then
                            do
                                local align = max(0,
                                    (ship:getPos() - playerShip:getPos()):normalize():dot(playerShip:getForward()))
                                if align * align >= 0.3 then
                                    insert(objects, ship)
                                end
                            end
                        end
                    end

                    -- Check to see if we should update visual jitter this pass
                    local barTweakVals = false
                    deltaSensorsTimer = deltaSensorsTimer + deltaTime
                    if deltaSensorsTimer > updateSensorsInterval then
                        -- Yes, update jitter this pass
                        barTweakVals = true
                        deltaSensorsTimer = 0
                    end

                    -- Initialize sensor bar values for this pass
                    for i = 1, barCount do
                        barHeightUp[i]   = 0
                        barHeightDown[i] = 0
                    end

                    -- Loop through all object close enough and in the sensor cone
                    -- Sum up all their values in each frequency band modified by distance and by dropoff from the cone's center
                    -- Multiply these values by the max onscreen size of the up/down bars to get each bar's height in pixels
                    for _, object in ipairs(objects) do
                        local emType = Enums.Emitters.None
                        local targetType = object:getType()
                        if targetType == Config:getObjectTypeByName("object_types", "Star") then
                            emType = Enums.Emitters.Star
                        elseif targetType == Config:getObjectTypeByName("object_types", "Planet") then
                            emType = Enums.Emitters.Planet
                        elseif targetType == Config:getObjectTypeByName("object_types", "Station") then
                            emType = Enums.Emitters.Station
                        elseif targetType == Config:getObjectTypeByName("object_types", "Ship") then
                            emType = Enums.Emitters.Ship
                        end

                        local dropoffDist = Config.gen.objectEmissionsDropoff[emType]

                        -- Get the distance from the player's ship to the object
                        local distance = playerShip:getDistance(object)

                        -- Get a number from 0 - 1 describing how directly the player's ship is looking at an object
                        local align = max(0,
                            (object:getPos() - playerShip:getPos()):normalize():dot(playerShip:getForward()))

                        -- Calculate signal dropoff as it reaches the player's ship's sensors
                        -- Reduce the strength of the signal itself based on distance (no need to go inverse-square for unnecessary "realism" here)
                        -- Reduce the strength of the received signal based on alignment of the sensor cone with the object
                        -- TODO: Modify strength & accuracy of received signal based on number & quality of ship's Sensor components
                        local dropoff = align * align * (1 - min(dropoffDist, distance) / dropoffDist)

                        for i = 1, barCount do
                            barHeightUp[i]   = barHeightUp[i] +
                                Config.gen.objectEmissions[i][emType] * barBase * dropoff / 100
                            barHeightDown[i] = barHeightDown[i] +
                                Config.gen.objectEmissions[i][emType] * barReflect * dropoff / 100
                        end
                    end

                    -- Display each sensor frequency bar
                    for i = 1, barCount do
                        if barTweakVals then
                            -- Add some jitter to the sensor bars
                            barTweak = rng:getUniformRange(-5, 5)
                        end

                        -- Add jitter constrained by the max height of the up and down (reflection) bars
                        local barHeightU = min(barBase, barHeightUp[i] + barTweak)
                        local barHeightD = min(barReflect, barHeightDown[i] + barTweak)

                        -- Get the highest bar ratio (greatest frequency bar value divided by the maximum possible bar height)
                        local barRatio = barHeightU / barBase
                        if barRatio > maxBarRatio then maxBarRatio = barRatio end

                        -- Finally, actually display all the sensor frequency bars
                        UI.DrawEx.Rect(xleft + ((i - 1) * (barWidth + 1)),
                            ybottom - barHeightU,
                            barWidth,
                            barHeightU,
                            Config.ui.color.meterBarBright)
                        UI.DrawEx.Rect(xleft + ((i - 1) * (barWidth + 1)),
                            ybottom,
                            barWidth,
                            barHeightD,
                            Config.ui.color.meterBarDark)
                    end
                end
            end

            -- *** TEMP: Audio FX test START ***
            if Config.audio.sounds.fxSensors then
                if not self.sensorSoundInstance then
                    -- start looping sound
                    self.sensorSoundInstance = Config.audio.sounds.fxSensors:play(maxBarRatio)
                else
                    local vol = maxBarRatio
                    -- prevent jitter
                    if vol < 0.1 then
                        vol = 0
                    end
                    -- adjust volume
                    self.sensorSoundInstance:setVolume(vol)
                end
            end
            -- *** TEMP: Audio FX test END ***
        end
    else
        -- stop sound
        if self.sensorSoundInstance then
            self.sensorSoundInstance:stop()
            self.sensorSoundInstance = nil
        end
    end
end

function HUD:drawTacticalMap(a)
    local cx, cy = self.sx / 2, self.sy / 2
    local radiusOuterRing = 70
    local radiusInnerRing = 44
    local radiusScaleRing = 10
    local mapHeight = 82

    -- Draw tactical map
    if GameState.ui.tacMapRange > Enums.HudTacMapRanges.Normal then
        UI.DrawEx.Ring(cx, self.sy - mapHeight, radiusOuterRing + 10, Config.ui.color.meterBarLight, true)
    end
    if GameState.ui.tacMapRange > Enums.HudTacMapRanges.Close then
        UI.DrawEx.Ring(cx, self.sy - mapHeight, radiusOuterRing + 5, Config.ui.color.meterBarLight, true)
    end
    UI.DrawEx.Ring(cx, self.sy - mapHeight, radiusOuterRing, Config.ui.color.meterBarLight, true)
    UI.DrawEx.Ring(cx, self.sy - mapHeight, radiusInnerRing, Config.ui.color.meterBarLight, false)

    UI.DrawEx.Line(cx, self.sy - 150, cx, self.sy - 12, Config.ui.color.meterBarLight, false)
    UI.DrawEx.Line(cx - radiusOuterRing, self.sy - 84, cx + radiusOuterRing, self.sy - 84, Config.ui.color.meterBarLight, false)

    UI.DrawEx.Line(cx - 48, self.sy - 130, cx, self.sy - 84, Config.ui.color.meterBarLight, false)
    UI.DrawEx.Line(cx + 48, self.sy - 130, cx, self.sy - 84, Config.ui.color.meterBarLight, false)

    -- Loop through nearby stations and ships and draw them on the tactical map
    local ix = self.sx / 2
    local iy = self.sy - mapHeight
    local r = 1.0
    local c = Color(1.0, 1.0, 1.0, 1.0)

    local camera = self.gameView.camera
    local player = self.player
    local playerShip = player:getControlling()
    local system = playerShip.parent
    if system and not playerShip:isShipDocked() then -- no need to update tactical map while docked at a space station
        local stations = system:getStations()
        local ships    = system:getShips()
        local maxDist  = Config.ui.general.rangeDistances[GameState.ui.tacMapRange]

        for _, station in ipairs(stations) do
            local dist = playerShip:getDistance(station)
            if not station:isDestroyed() and dist <= maxDist then
                local pos = station:getPos()
                local ndc = camera:worldToNDC(pos)
                local x = ix + math.sin(max(-1.57, min(1.57, ndc.x))) *  radiusOuterRing * ndc.z * dist / maxDist
                local y = iy + math.cos(max(-1.57, min(1.57, ndc.x))) * -radiusOuterRing * ndc.z * dist / maxDist

                local disp = Config.game.dispoNeutral -- disposition to neutral by default
                if station:hasAttackable() and station:isAttackable() then disp = station:getDisposition(playerShip) end
                -- local c = target:getDispositionColor(disp) -- this version is preserved for future changes (esp. faction)
                local c = Disposition.GetColor(disp)

                r = 1.5
                UI.DrawEx.Circle(x, y, r, c)
            end
        end
        for _, ship in ipairs(ships) do
            local dist = playerShip:getDistance(ship)
            if ship ~= playerShip and not ship:isDestroyed() and dist <= maxDist then
                local pos = ship:getPos()
                local ndc = camera:worldToNDC(pos)
                local x = ix + math.sin(max(-1.7, min(1.7, ndc.x))) *  radiusOuterRing * ndc.z * dist / maxDist
                local y = iy + math.cos(max(-1.7, min(1.7, ndc.x))) * -radiusOuterRing * ndc.z * dist / maxDist

                local disp = Config.game.dispoNeutral -- disposition to neutral by default
                if ship:hasAttackable() and ship:isAttackable() then disp = ship:getDisposition(playerShip) end
                -- local c = target:getDispositionColor(disp) -- this version is preserved for future changes (esp. faction)
                local c = Disposition.GetColor(disp)

                UI.DrawEx.Point(x, y, 256, c)
            end
        end
    end
end

function HUD:drawDynamicLightsFlag(a)
    local cx, cy = self.sx / 2, self.sy / 2
    local fontsize = 14

    if GameState.render.pulseLights then
        UI.DrawEx.Circle(cx - 90, self.sy - 30, 9, Config.ui.color.meterBarLight, true)
        HUD:drawHudText("UbuntuBold", fontsize + 2, cx - 97, self.sy - 38,
            "L", Config.ui.color.borderBright)
    else
        UI.DrawEx.Ring(cx - 90, self.sy - 30, 10, Config.ui.color.meterBarLight, false)
        HUD:drawHudText("Ubuntu", fontsize, cx - 96, self.sy - 37,
            "L", Config.ui.color.meterBarLight)
    end
end

local function getPosObject(def)
    local object = {}
    object.c = def.c
    object.a = def.a
    object.curve = def.curve
    object.size1 = def.size1
    object.size2 = def.size2
    object.bx = def.bx
    object.by = def.by
    object.offset = def.offset
    return object
end

function HUD:drawTargets(a)
    deltaTimer = deltaTimer + deltaTime
    if deltaTimer > lastTargetsUpdate + updateTargetsInterval then
        if not GameState.ui.showTrackers then return end
        local camera = self.gameView.camera

        local cTarget = Color(0.5, 1.0, 0.1, 1.0 * a)
        local cLock = Color(1.0, 0.5, 0.1, 1.0 * a)

        local player = self.player
        local playerShip = player:getControlling()
        local playerTarget = playerShip:getTarget()

        local closest = nil
        local minDist = 128
        local center = Vec2f(self.sx / 2, self.sy / 2)
        targetsHudPositions = {}

        for i = 1, #self.targets.tracked do
            local target = self.targets.tracked[i]
            local targetDistance = target:getDistance(playerShip)

            if target and targetDistance and target ~= playerShip then
                -- if target is out of trackingRange
                if targetDistance > GameState.ui.maxTrackingRange then break end

                if target:getTrackable() then
                    local pos = target:getPos()
                    local ndc = camera:worldToNDC(pos)
                    local ray = camera:ndcToRay(ndc, targetDistance)
                    ndc = camera:worldToNDC(pos)
                    local ndcMax = max(abs(ndc.x), abs(ndc.y))

                    local system = player:getRoot()
                    local hit = system.physics:rayCast(ray).body
                    local alphaOverwrite = nil

                    if hit ~= nil then
                        while hit:getParentBody() ~= nil do hit = hit:getParentBody() end
                        local hitEnt = Entity.fromRigidBody(hit)
                        --
                        if hitEnt ~= target then
                            alphaOverwrite = math.max(0, math.min(GameState.ui.trackerObjectOcclusion, 1.0))
                        end
                    end

                    -- local disp = target:getOwnerDisposition(player) -- might need to switch back to this version
                    local disp = Config.game.dispoNeutral -- disposition to neutral by default
                    if target:hasAttackable() and target:isAttackable() then disp = target:getDisposition(playerShip) end
                    -- local c = target:getDispositionColor(disp) -- this version is preserved for future changes (esp. faction)
                    local c = Disposition.GetColor(disp)
                    c.a = alphaOverwrite or (1 - (targetDistance / GameState.ui.maxTrackingRange))

                    if ndcMax <= 1.0 and ndc.z > 0 then
                        do
                            -- Get tracker box extents based on object size, and adjust inward slightly
                            local bx1, by1, bsx, bsy = camera:entityToScreenRect(target)
                            local bx2, by2 = bx1 + bsx, by1 + bsy

                            local function drawAttackable()
                                table.insert(targetsHudPositions, getPosObject({
                                    c = c,
                                    a = c.a,
                                    curve = 0.2,
                                    size1 = 4,
                                    size2 = 4,
                                    bx = bx2,
                                    by = by1,
                                    offset = 0.125
                                }))
                                table.insert(targetsHudPositions, getPosObject({
                                    c = c,
                                    a = c.a,
                                    curve = 0.2,
                                    size1 = 4,
                                    size2 = 4,
                                    bx = bx1,
                                    by = by1,
                                    offset = 0.375
                                }))
                                table.insert(targetsHudPositions, getPosObject({
                                    c = c,
                                    a = c.a,
                                    curve = 0.2,
                                    size1 = 4,
                                    size2 = 4,
                                    bx = bx1,
                                    by = by2,
                                    offset = 0.625
                                }))
                                table.insert(targetsHudPositions, getPosObject({
                                    c = c,
                                    a = c.a,
                                    curve = 0.2,
                                    size1 = 4,
                                    size2 = 4,
                                    bx = bx2,
                                    by = by2,
                                    offset = 0.875
                                }))
                            end

                            local function drawPlayerTarget()
                                table.insert(targetsHudPositions, getPosObject({
                                    c = cLock,
                                    a = a,
                                    curve = 0.3,
                                    size1 = 12,
                                    size2 = 12,
                                    bx = bx2,
                                    by = by1,
                                    offset = 0.125
                                }))
                                table.insert(targetsHudPositions, getPosObject({
                                    c = cLock,
                                    a = a,
                                    curve = 0.3,
                                    size1 = 12,
                                    size2 = 12,
                                    bx = bx1,
                                    by = by1,
                                    offset = 0.375
                                }))
                                table.insert(targetsHudPositions, getPosObject({
                                    c = cLock,
                                    a = a,
                                    curve = 0.3,
                                    size1 = 12,
                                    size2 = 12,
                                    bx = bx1,
                                    by = by2,
                                    offset = 0.625
                                }))
                                table.insert(targetsHudPositions, getPosObject({
                                    c = cLock,
                                    a = a,
                                    curve = 0.3,
                                    size1 = 12,
                                    size2 = 12,
                                    bx = bx2,
                                    by = by2,
                                    offset = 0.875
                                }))
                            end

                            local function drawTarget()
                                table.insert(targetsHudPositions, getPosObject({
                                    c = cTarget,
                                    a = a,
                                    curve = 0.2,
                                    size1 = 8,
                                    size2 = 8,
                                    bx = bx2,
                                    by = by1,
                                    offset = 0.125
                                }))
                                table.insert(targetsHudPositions, getPosObject({
                                    c = cTarget,
                                    a = a,
                                    curve = 0.2,
                                    size1 = 8,
                                    size2 = 8,
                                    bx = bx1,
                                    by = by1,
                                    offset = 0.375
                                }))
                                table.insert(targetsHudPositions, getPosObject({
                                    c = cTarget,
                                    a = a,
                                    curve = 0.2,
                                    size1 = 8,
                                    size2 = 8,
                                    bx = bx1,
                                    by = by2,
                                    offset = 0.625
                                }))
                                table.insert(targetsHudPositions, getPosObject({
                                    c = cTarget,
                                    a = a,
                                    curve = 0.2,
                                    size1 = 8,
                                    size2 = 8,
                                    bx = bx2,
                                    by = by2,
                                    offset = 0.875
                                }))
                            end

                            local type = Config:getObjectInfo("object_types", target:getType())
                            local renderDistance = GameState.ui.trackerBracketingRenderDistances[type] or 25000

                            -- Draw rounded box corners
                            if targetDistance <= renderDistance or playerTarget == target then
                                if target:hasAttackable() and target:isAttackable() then
                                    -- Innermost box shows trackable object's disposition to player
                                    -- (red = enemy, blue = neutral, green = friendly)
                                    drawAttackable()
                                end

                                if playerTarget == target then
                                    drawPlayerTarget()
                                end

                                if self.target == target then
                                    drawTarget()
                                end
                            elseif target:hasAttackable() and target:isAttackable() and targetDistance >= renderDistance then
                                table.insert(targetsHudPositions, { bx1 = bx1, by1 = by1, bx2 = bx2, by2 = by2, c = c })

                                if playerTarget == target then
                                    drawPlayerTarget()
                                end

                                if self.target == target then
                                    drawTarget()
                                end
                            end

                            -- Draw target name
                            if playerTarget == target then
                                local targetName = target:getName()
                                if target:getType() == Config:getObjectTypeByName("object_types", "Planet") then
                                    targetName = "Planet " .. target:getName()
                                elseif target:getType() == Config:getObjectTypeByName("object_types", "Asteroid") then
                                    targetName = "Asteroid " .. target:getName()
                                elseif target:getType() == Config:getObjectTypeByName("object_types", "Station") then
                                    targetName = "Station " .. target:getName()
                                elseif target:getType() == Config:getObjectTypeByName("object_types", "Jumpgate") then
                                    targetName = "Jumpgate " .. target:getName()
                                elseif target:getType() == Config:getObjectTypeByName("object_types", "Ship") then
                                    if target.usesBoost then
                                        targetName = targetName .. " [Ace]"
                                    end
                                end
                                local tcr = 1
                                local tcg = 1
                                local tcb = 1
                                if target:isDestroyed() then
                                    tcr = 0
                                    tcg = 0
                                    tcb = 0
                                end
                                table.insert(targetsHudPositions,
                                    {
                                        bx1 = bx1,
                                        by1 = by1,
                                        bx2 = bx2,
                                        by2 = by2,
                                        tcr = tcr,
                                        tcg = tcg,
                                        tcb = tcb,
                                        a = a,
                                        targetName = targetName
                                    })
                            end
                        end

                        -- TEMP: Draw target health bar
                        --[[
                        if playerTarget == target and target:isAlive() and not target:isDestroyed() then
                            local targetHealthPct = target:mgrHullGetHealthPercent()
                            if targetHealthPct > 0.0 then
                                local targetHealthCI = math.min(50, math.floor((targetHealthPct / 2.0) + 0.5) + 1)
                                UI.DrawEx.RectOutline(bx1 + 2, by2 - 3, (bx2 - bx1) - 6, 8, Config.ui.color.borderBright)
                                UI.DrawEx.Rect(bx1 + 3, by2 - 1, (bx2 - bx1) - 8, 4,
                                    Config.ui.color.healthColor[targetHealthCI])
                            end
                        end
                        --]]

                        local ss = camera:ndcToScreen(ndc)
                        local dist = ss:distance(center)
                        if disp < 0.5 and dist < minDist then
                            closest = target
                            minDist = dist
                        end
                    else
                        ndc.x = ndc.x / ((1 + 16 / camera.sx) * ndcMax)
                        ndc.y = ndc.y / ((1 + 16 / camera.sy) * ndcMax)
                        local x = (ndc.x + 1) / 2 * camera.sx
                        local y = (-ndc.y + 1) / 2 * camera.sy
                        if disp < 0.0 then
                            c.a = c.a * 0.5
                            UI.DrawEx.Point(x, y, 64, c)
                        end
                    end
                end
            end
            ::skipTarget::
        end
        lastTargetsUpdate = deltaTimer
        self.target = closest
    end

    for index, targetHud in ipairs(targetsHudPositions) do
        if targetHud.bx1 and not targetHud.tcr then
            UI.DrawEx.Point(targetHud.bx2 - ((targetHud.bx2 - targetHud.bx1) / 2),
                targetHud.by2 - ((targetHud.by2 - targetHud.by1) / 2), 128, targetHud.c)
        elseif targetHud.tcr then
            UI.DrawEx.TextAdditive(
                "UbuntuBold",
                targetHud.targetName,
                14,
                (targetHud.bx1 + targetHud.bx2) / 2 - targetHud.targetName:len() / 2 + 1, targetHud.by1 - 30 + 1,
                targetHud.targetName:len(), 20,
                1 - targetHud.tcr, 1 - targetHud.tcg, 1 - targetHud.tcb, targetHud.a,
                0.5, 0.5
            )
            UI.DrawEx.TextAlpha(
                "UbuntuBold",
                targetHud.targetName,
                14,
                (targetHud.bx1 + targetHud.bx2) / 2 - targetHud.targetName:len() / 2, targetHud.by1 - 30,
                targetHud.targetName:len(), 20,
                targetHud.tcr, targetHud.tcg, targetHud.tcb, targetHud.a,
                0.5, 0.5
            )
        else
            UI.DrawEx.Wedge(targetHud.bx, targetHud.by, targetHud.size1, targetHud.size2, targetHud.offset,
                targetHud.curve, targetHud.c, targetHud.a)
        end
    end
end

function HUD:drawDirPredict(a)
    local playerShip = self.player:getControlling()
    local target = playerShip:getTarget()

    if not target or target:isDestroyed() then return end

    local camera = self.gameView.camera
    local center = Vec2f(self.sx / 2, self.sy / 2)

    do -- Direction indicator
        local r = 96
        local pos = target:getPos()
        local ndc = camera:worldToNDC(pos)
        local ndcMax = max(abs(ndc.x), abs(ndc.y))

        -- NOTE: invert direction arrow when target in rear hemisphere relative to player view
        if ndc.z <= 0 then ndc:idivs(-ndcMax) end

        local ss = camera:ndcToScreen(ndc)
        local dir = ss - center
        local dist = dir:length()

        if dist > 1 then
            dir:inormalize()
            ss = center + dir:scale(r)
            local a = a * (1.0 - exp(-max(0.0, dist / (r + 16) - 1.0)))
            UI.DrawEx.Arrow(ss, dir:scale(10), Color(1.0, 0.7, 0.3, a))
        end
    end

    -- Predictive impact point
    -- Takes into account player's movement, target's movement,
    -- and the speed of the currently selected weapon/projectile
    -- TODO: change reference to Config.gen.compTurretPulseStats.range from App.lua when multiple weapon types are available
    local range = playerShip:getPos():distance(target:getPos())
    if target:hasAttackable() and target:isAttackable() and range < Config.gen.compTurretPulseStats.range then
        if playerShip.socketSpeedMax > 0 then
            local tHit, pHit = Math.Impact(
                playerShip:getPos(),
                target:getPos(),
                playerShip:getVelocity(),
                target:getVelocity(),
                playerShip.socketSpeedMax)

            if tHit then
                local ndc = camera:worldToNDC(pHit)
                local ndcMax = max(abs(ndc.x), abs(ndc.y))
                if ndcMax <= 1 and ndc.z > 0 then
                    local ss = camera:ndcToScreen(ndc)
                    UI.DrawEx.Ring(ss.x, ss.y, 10, Color(1.0, 0.3, 0.3, a), true)
                end
            end
        end
    end
end

function HUD:drawReticle(a)
    local cx, cy = self.sx / 2, self.sy / 2
    do     -- Reticle
        do -- Central crosshair
            local c = Config.ui.color.reticle
            local r1 = 24
            local r2 = 36
            local n = 3
            for i = 0, n - 1 do
                local angle = -(Math.Pi2 + (i / n) * Math.Tau)
                local dx, dy = cos(angle), sin(angle)
                UI.DrawEx.Line(cx + r1 * dx, cy + r1 * dy, cx + r2 * dx, cy + r2 * dy, c, true)
            end
        end

        -- Flight mode cursor
        if not GameState.panelActive then
            local c = Config.ui.color.ctrlCursor
            -- Local yaw, pitch = ShipBindings.Yaw:get(), ShipBindings.Pitch:get()
            local x = cx + 0.5 * self.sx * self.aimX
            local y = cy - 0.5 * self.sy * self.aimY

            local csize = 16
            UI.DrawEx.Ring(x, y, csize, c, false)
            UI.DrawEx.Line(x - csize, y, x - 2, y, c, true)
            UI.DrawEx.Line(x, y - csize, x, y - 2, c, true)
            UI.DrawEx.Line(x + csize, y, x + 2, y, c, true)
            UI.DrawEx.Line(x, y + csize, x, y + 2, c, true)
        end
    end
end

function HUD:drawPlayerSystemIntegrity(a)
    local x, y, sx, sy    = self:getRectGlobal()
    local cx, cy          = sx / 2, sy / 2
    local playerShip      = self.player:getControlling()
    local playerZoom      = playerShip:getRadius() / (playerShip:getScale() / 4)
    local playerShieldPct = playerShip:mgrShieldGetStrengthPercent()
    local playerArmorPct  = playerShip:mgrArmorGetHealthPercent()
    local playerHealthPct = playerShip:mgrHullGetHealthPercent()
    local playerCapChgPct = playerShip:mgrCapacitorGetChargePercent()
    local playerHealthCI  = math.min(50, math.floor((playerHealthPct / 2.0) + 0.5) + 1)

    local hc              = Color(1, 1, 1, 1)
    hc.r                  = Config.ui.color.healthColor[playerHealthCI].r
    hc.g                  = Config.ui.color.healthColor[playerHealthCI].g
    hc.b                  = Config.ui.color.healthColor[playerHealthCI].b
    hc.a                  = 0.7

    -- ** Uncomment if we need to retrieve the values used to define the viewing angle of holograms **
    -- if not GameState.paused then
    -- local radius, mass = playerShip:getRadius(), playerShip:getMass()
    -- local yaw, pitch = ShipBindings.Yaw:get(), ShipBindings.Pitch:get()
    -- Log.Debug("x = %d, y = %d, sx = %d, sy = %d", x, y, sx, sy)
    -- Log.Debug("mass = %s, radius = %3.2f, yaw = %3.2f, pitch = %3.2f", mass, radius, yaw, pitch)
    -- Log.Debug("mass = %s, radius = %3.2f, radius / 1.7 = %3.2f", mass, radius, radius / 1.7)
    -- end

    -- Draw text of player ship name
    HUD:drawHudTextDouble(164, sy - 280, Config.ui.color.meterBarLight, hudFontSize, 0.5, playerShip:getName())

    -- Draw hologram of player ship on a grid background
--    UI.DrawEx.Grid(114, sy - 180, 100, 100, Config.ui.color.meterBarLight)
    UI.DrawEx.Hologram(playerShip.mesh, 34, sy - 260, 260, 260, Config.ui.color.hologram, playerZoom / 1.7, -1.57, 0.0)

    -- Draw the current percentage statuses of key player ship systems as concentric rings
    for str = floor(playerShieldPct / 10), 1, -1 do
        UI.DrawEx.RingDim(164, sy - 130, 110 + (str - 1), Config.ui.color.shieldStrength)
    end
    for str = floor(playerArmorPct / 10), 1, -1 do
        UI.DrawEx.RingDim(164, sy - 130,  95 + (str - 1), Config.ui.color.armorIntegrity)
    end
    for str = floor(playerHealthPct / 10), 1, -1 do
        UI.DrawEx.RingDim(164, sy - 130,  80 + (str - 1), Config.ui.color.hullIntegrity)
    end
    for str = floor(playerCapChgPct / 10), 1, -1 do
        UI.DrawEx.RingDim(164, sy - 130,  65 + (str - 1), Config.ui.color.capacitorEnergy)
    end

    -- Also draw player ship hull, armor, and shield strengths in text form
    local text = ""
    text = format("%d%%", playerHealthPct)
    HUD:drawHudTextDouble(164, sy - 54, Config.ui.color.meterBarLight, hudFontSize - 2, 0.5, text)
    text = format("%d%%", playerArmorPct)
    HUD:drawHudTextDouble(164, sy - 38, Config.ui.color.meterBarLight, hudFontSize - 2, 0.5, text)
    text = format("%d%%", playerShieldPct)
    HUD:drawHudTextDouble(164, sy - 22, Config.ui.color.meterBarLight, hudFontSize - 2, 0.5, text)
end

function HUD:drawTargetSystemIntegrity(a)
    local playerShip = self.player:getControlling()
    local target = playerShip:getTarget()
    if target and target:isAlive() and not target:isDestroyed() then
        local x, y, sx, sy = self:getRectGlobal()
        local cx, cy = sx / 2, sy / 2
        local targetRangeText = ""
        if playerShip:getDistance(target) >= 1000 then
            targetRangeText = format("Range: %d km", floor(playerShip:getDistance(target) / 1000 + 0.5))
        else
            targetRangeText = format("Range: %d m", floor(playerShip:getDistance(target) + 0.5))
        end
        local targetName = target:getName()
        local targetHealthPct = target:mgrHullGetHealthPercent()
        if targetHealthPct > 0.0 then
            local targetShieldPct = target:mgrShieldGetStrengthPercent()
            local targetArmorPct  = target:mgrArmorGetHealthPercent()
            local targetHealthCI  = math.min(50, math.floor((targetHealthPct / 2.0) + 0.5) + 1)
            local targetZoom      = target:getRadius() / (target:getScale() / 4)
            local targetZoomAdj   = targetZoom

            local hc              = Color(1, 1, 1, 1)
            hc.r                  = Config.ui.color.healthColor[targetHealthCI].r
            hc.g                  = Config.ui.color.healthColor[targetHealthCI].g
            hc.b                  = Config.ui.color.healthColor[targetHealthCI].b
            hc.a                  = 0.7

            if target:getType() == Config:getObjectTypeByName("object_types", "Ship") then
                targetZoomAdj = targetZoom / 1.7
                if target.usesBoost then
                    targetName = targetName .. " [Ace]"
                end
            end
            if target:getType() == Config:getObjectTypeByName("object_types", "Station") then
                targetZoomAdj = 26 -- probably station radius (default: 100) / station scale (default: 4)
                targetName = "Station " .. target:getName()
            end

            -- Draw text of target name
            HUD:drawHudTextDouble(sx - 160, sy - 280, Config.ui.color.meterBarLight, hudFontSize, 0.5, targetName)

            -- Draw hologram of target entity on a grid background
--            UI.DrawEx.Grid(sx - 206, sy - 180, 100, 100, Config.ui.color.meterBarLight)
            UI.DrawEx.Hologram(target.mesh, sx - 287, sy - 260, 260, 260, Config.ui.color.hologram, targetZoomAdj, -1.57,
                0.0)

            -- Draw the current percentage statuses of key target ship systems as concentric rings
            for str = floor(targetShieldPct / 10), 1, -1 do
                UI.DrawEx.RingDim(sx - 156, sy - 130, 110 + (str - 1), Config.ui.color.shieldStrength)
            end
            for str = floor(targetArmorPct / 10), 1, -1 do
                UI.DrawEx.RingDim(sx - 156, sy - 130,  95 + (str - 1), Config.ui.color.armorIntegrity)
            end
            for str = floor(targetHealthPct / 10), 1, -1 do
                UI.DrawEx.RingDim(sx - 156, sy - 130,  80 + (str - 1), Config.ui.color.hullIntegrity)
            end

            -- Also draw target ship hull, armor, and shield strengths in text form
            local text = ""
            text = format("%d%%", targetHealthPct)
            HUD:drawHudTextDouble(sx - 156, sy - 54, Config.ui.color.meterBarLight, hudFontSize - 2, 0.5, text)
            text = format("%d%%", targetArmorPct)
            HUD:drawHudTextDouble(sx - 156, sy - 38, Config.ui.color.meterBarLight, hudFontSize - 2, 0.5, text)
            text = format("%d%%", targetShieldPct)
            HUD:drawHudTextDouble(sx - 156, sy - 22, Config.ui.color.meterBarLight, hudFontSize - 2, 0.5, text)
        end
    end
end

function HUD:drawDockPrompt(a)
    local x, y, sx, sy = self:getRectGlobal()
    local cx, cy = sx / 2, sy / 2
    local dockText = nil

    if dockingAllowed then
        dockText = "Press F to Dock" -- TODO: connect Docking input to bindings
    else
        dockText = "Docking is refused at this Station"
    end

    UI.DrawEx.TextAdditive(
        "NovaMono",
        dockText,
        24,
        cx, cy - 68, 1, 1,
        0, 0, 0, self.dockPromptAlpha * a,
        0.5, 0.5
    )
    UI.DrawEx.TextAdditive(
        "NovaMono",
        dockText,
        24,
        cx, cy - 68, 1, 1,
        1, 1, 1, self.dockPromptAlpha * a,
        0.5, 0.5
    )
end

function HUD:onInput(state)
    if not GameState.paused and not GameState.panelActive then
        local camera = self.gameView.camera
        camera:push()

        if camera.modRadius then
            camera:modRadius(exp(-0.1 * CameraBindings.Zoom:get()))
        end
        -- camera:modYaw(0.005 * CameraBindings.Yaw:get())     -- only works when cameraOrbit is the current camera
        -- camera:modPitch(0.005 * CameraBindings.Pitch:get()) -- only works when cameraOrbit is the current camera

        -- HUD control: Select a weapon group
        if Input:isPressed(Button.KeyboardKey1) and GameState.player.weaponGroup ~= 1 then
            GameState.player.weaponGroup = 1
        elseif Input:isPressed(Button.KeyboardKey2) and GameState.player.weaponGroup ~= 2 then
            GameState.player.weaponGroup = 2
        elseif Input:isPressed(Button.KeyboardKey3) and GameState.player.weaponGroup ~= 3 then
            GameState.player.weaponGroup = 3
        elseif Input:isPressed(Button.KeyboardKey4) and GameState.player.weaponGroup ~= 4 then
            GameState.player.weaponGroup = 4
        elseif Input:isPressed(Button.KeyboardKey5) and GameState.player.weaponGroup ~= 5 then
            GameState.player.weaponGroup = 5
        elseif Input:isPressed(Button.KeyboardKey6) and GameState.player.weaponGroup ~= 6 then
            GameState.player.weaponGroup = 6
        elseif Input:isPressed(Button.KeyboardKey7) and GameState.player.weaponGroup ~= 7 then
            GameState.player.weaponGroup = 7
        elseif Input:isPressed(Button.KeyboardKey8) and GameState.player.weaponGroup ~= 8 then
            GameState.player.weaponGroup = 8
        end

        -- HUD control: Select a tactical map range (close, normal, distant)
        if Input:isPressed(Button.KeyboardNumpad1) then
            GameState.ui.tacMapRange = 1
        elseif Input:isPressed(Button.KeyboardNumpad2) then
            GameState.ui.tacMapRange = 2
        elseif Input:isPressed(Button.KeyboardNumpad3) then
            GameState.ui.tacMapRange = 3
        end

        -- Process player ship control actions
        local e = self.player:getControlling()
        if not e:isDestroyed() then
            self:controlThrust(e)
            self:controlTurrets(e)
            self:controlBays(e)
            self:controlTargetLock(e)
        end
        camera:pop()

        if self.dockable then
            -- Log.Debug("%s %s is dockable = %s", Config:getObjectInfo("object_types", self.dockable:getType()),
            -- self.dockable:getName(), self.dockable:isDockable())
            if self.dockable:isDockable() and not self.dockable:isBanned(e) then
                if ShipBindings.Dock:get() > 0 then
                    -- TODO: migrate this action outside the HUD
                    e:pushAction(Actions.DockAt(self.dockable))
                    self.dockable = nil
                end
            end
        end
    end
end

function HUD:onUpdate(state)
    if not GameState.paused then
        if Input:isPressed(Bindings.ToggleHUD) then
            GameState.ui.hudStyle = GameState.ui.hudStyle + 1
            if GameState.ui.hudStyle > Enums.HudStyles.Tight then
                GameState.ui.hudStyle = Enums.HudStyles.None
            end
        end

        if Input:isPressed(Bindings.ToggleSensors) then
            GameState.ui.sensorsDisplayed = not GameState.ui.sensorsDisplayed
        end

        self.targets:update()
        self.dockables:update()

        self.dockable = HUD:getDockable(self)

        hudFontSize = 14 + (floor(self.sx / 900) - 1) * 2

        lockTimer = lockTimer + state.dt
        deltaTime = state.dt

        local f = 1.0 - exp(-state.dt * 8.0)
        local alphaT = 0
        if self.dockable then
            if self.dockable:isDockable() and not self.dockable:isBanned(self.player:getControlling()) then
                dockingAllowed = true
                alphaT = 1
            else
                dockingAllowed = false
                if not self.dockable:isDestroyed() then
                    alphaT = 1
                else
                    alphaT = 0
                end
            end
        end
        self.dockPromptAlpha = Math.Lerp(self.dockPromptAlpha, alphaT, f)
    end

    if Config.audio.fxSensors then
        if GameState.paused then
            if Config.audio.fxSensors:IsPlaying() then
                Config.audio.fxSensors:Pause()
            end
        else
            if GameState.ui.sensorsDisplayed then
                if Config.audio.fxSensors:IsPaused() then
                    Config.audio.fxSensors:Resume()
                end
            else
                if Config.audio.fxSensors:IsPlaying() then
                    Config.audio.fxSensors:Pause()
                end
            end
        end
    end
end

function HUD:getDockable(self)
    local dockableObj = nil

    local pPos        = self.player:getControlling():getPos()
    local pRad        = self.player:getControlling():getRadius()
    self.dockable     = nil
    for i = 1, #self.dockables.tracked do
        local dockable = self.dockables.tracked[i]
        if Config:getObjectInfo("object_types", dockable:getType()) ~= "Planet" then -- player's ship can't dock at planets
            local dPos = dockable:getPos()
            local dRad = dockable:getRadius()
            local dist = pPos:distance(dPos) - pRad - dRad
            if dist < Config.game.dockRange then
                -- Return the Entity instance of the first dockable object found (might not be closest if several are within range)
                dockableObj = dockable
                break
            end
        end
    end

    return dockableObj
end

function HUD:onDraw(focus, active)
    local playerShip = self.player:getControlling()
    if playerShip:isAlive() then
        if GameState.ui.hudStyle == Enums.HudStyles.Minimal then
            self:drawTargets(self.enabled)
            self:drawReticle(self.enabled)
            self:drawDirPredict(self.enabled)
        elseif GameState.ui.hudStyle ~= Enums.HudStyles.None then
            self:drawSystemText(self.enabled)
            self:drawTargetText(self.enabled)
            self:drawTargetMission(self.enabled)
            self:drawTargetType(self.enabled)
            self:drawTargetRange(self.enabled)
            self:drawTargetSubtype(self.enabled)
            self:drawTargetSpeed(self.enabled)
--            self:drawTargetShieldsHullArmor(self.enabled)
--            self:drawPlayerShieldsHullArmor(self.enabled)
            self:drawMissilesLeft(self.enabled)
            self:drawPlayerSpeed(self.enabled)
            self:drawChaffLeft(self.enabled)
            self:drawLockWarning(self.enabled)
            self:drawWeaponGroups(self.enabled)
            self:drawPowerDistro(self.enabled)
            self:drawSensors(self.enabled)
            self:drawTacticalMap(self.enabled)
            self:drawDynamicLightsFlag(self.enabled)
            self:drawPlayerSystemIntegrity(self.enabled)
            self:drawTargetSystemIntegrity(self.enabled)
            self:drawTargets(self.enabled)
            self:drawReticle(self.enabled)
            self:drawDirPredict(self.enabled)
        end

        self:drawDockPrompt(self.enabled)
    end
end

function HUD:onDrawIcon(iconButton, focus, active)
    -- Draw Flight Mode icon
    local borderColor = iconButton == active
        and Config.ui.color.controlActive
        or iconButton == focus
        and Config.ui.color.controlFocused
        or Config.ui.color.control
    local contentColor = self:isEnabled()
        and Config.ui.color.controlFocused
        or Config.ui.color.control

    local x, y, sx, sy = iconButton:getRectGlobal()
    UI.DrawEx.RectOutline(x, y, sx, sy, borderColor)

    local cx = x + sx / 2
    local w1y, w1sx, w1sy = 10, 10, 8
    local w2y, w2sx, w2sy = 0, 5, 4
    local ty, by = y + 8, y + sy - 12
    UI.DrawEx.Line(cx, ty, cx, by, contentColor, false)
    UI.DrawEx.Line(cx + 2, ty + w1y, cx + w1sx, ty + w1y + w1sy, contentColor, false)
    UI.DrawEx.Line(cx - 2, ty + w1y, cx - w1sx, ty + w1y + w1sy, contentColor, false)
    UI.DrawEx.Line(cx + 2, by, cx + w2sx, by + w2y + w2sy, contentColor, false)
    UI.DrawEx.Line(cx - 2, by, cx - w2sx, by + w2y + w2sy, contentColor, false)
end

function HUD:onEnable()
    -- TODO : Wtf does this do? Who wrote this?? WHY. [<-- this comment was from Josh or Adam]
    local pCamera = self.gameView.camera
    local camera = self.gameView.camera

    camera:warp()
    camera:lerpFrom(pCamera.pos, pCamera.rot)

    -- Set the mouse position when the Flight mode HUD is activated to the center of the game window
    GameState.render.gameWindow:cursor():setGrabMode(CursorGrabMode.Confined)
    local size = GameState.render.gameWindow:size()
    Input:setCursorPosition(size.x / 2, size.y / 2)
    GameState.render.gameWindow:cursor():setGrabMode(CursorGrabMode.None)
end

function HUD:controlThrust(e)
    -- TODO: Should this really be here in HUD.lua?
    if not e:hasThrustController() then return end
    local c = e:getThrustController()

    -- Create a small (square) dead zone in the center of the aiming reticle
    -- TODO: make dead zone circular and a sloping cutoff instead of sharp
    local yaw = ShipBindings.Yaw:get()
    if abs(yaw) < 0.004 then yaw = 0 end
    local pitch = ShipBindings.Pitch:get() -- make negative if ShipBindings.Pitch is not :invert()
    if abs(pitch) < 0.008 then pitch = 0 end

    c:setThrust(
        ShipBindings.ThrustZ:get(),
        ShipBindings.ThrustX:get() * 0.5,
        ShipBindings.ThrustY:get(),
        yaw,
        pitch,
        ShipBindings.Roll:get() * 0.5,
        ShipBindings.Boost:get())
    self.aimX = c.yaw
    self.aimY = c.pitch
    -- Log.Debug("yaw = %f, pitch = %f", c.yaw, c.pitch)
end

function HUD:controlTurrets(e)
    -- TODO: Should this really be here in HUD.lua?
    local targetPos, targetVel
    local target = e:getTarget()

    if target and target:getOwnerDisposition(self.player) <= 0.0 then
        targetPos = target:getPos()
        targetVel = target:getVelocity()
    end

    local firing   = ShipBindings.Fire:get() > 0 and 1 or 0
    local camera   = self.gameView.camera
    local ndc      = Vec3f(self.aimX, self.aimY)
    local fallback = camera:mouseToRay(1):getPoint(e.socketRangeMin)

    -- Compute a firing solution separately for each turret to support different projectile velocities & ranges
    for turret in e:iterSocketsByType(SocketType.Turret) do
        if Config.game.autoTarget and targetPos then
            turret:aimAtTarget(target, fallback)
        else
            turret:aimAt(fallback)
        end
        turret.firing = firing
    end
end

function HUD:controlBays(e)
    -- TODO: Should this really be here in HUD.lua?
    local targetPos, targetVel
    local target = e:getTarget()

    if target and target:getOwnerDisposition(self.player) <= 0.0 then
        targetPos = target:getPos()
        targetVel = target:getVelocity()
    end

    local firing   = ShipBindings.Fire:get() > 0 and 1 or 0
    local camera   = self.gameView.camera
    local ndc      = Vec3f(self.aimX, self.aimY)
    local fallback = camera:mouseToRay(1):getPoint(e.socketRangeMin)

    -- Compute a firing solution separately for each turret to support different projectile velocities & ranges
    for bay in e:iterSocketsByType(SocketType.Bay) do
        if Config.game.autoTarget and targetPos then
            bay:aimAtTarget(target, fallback)
        else
            bay:aimAt(fallback)
        end
        bay.firing = firing
    end
end

function HUD:controlTargetLock(e)
    -- Lock target under the player ship's reticle
    if ShipBindings.LockTarget:get() > 0.5 then e:setTarget(self.target) end

    -- Clear any currently locked target
    if ShipBindings.ClearTarget:get() > 0.5 then e:setTarget(nil) end

    -- Lock the nearest ship that is currently targeting the player's ship
    if ShipBindings.NearestTarget:get() > 0.5 then
        local player = self.player
        local playerShip = player:getControlling()
        local nearestTargeting = nil
        local nearestDistance = 1e10
        for _, targetingShip in ipairs(GameState.world.currentSystem.ships) do
            if targetingShip:getTarget() == playerShip then
                local targetingDistance = targetingShip:getDistance(playerShip)
                if targetingDistance < nearestDistance then
                    nearestTargeting = targetingShip
                    nearestDistance = targetingDistance
                end
            end
        end
        if nearestTargeting then e:setTarget(nearestTargeting) end
    end
end

function HUD:drawHudText(font, fontsize, x, y, text, c)
    UI.DrawEx.TextAdditive(
        font,
        text,
        fontsize,
        x, y, fontsize, fontsize,
        c.r, c.g, c.b, 1.0,
        0.5, 0.5
    )
end

function HUD:drawHudTextDouble(x, y, color, size, horJust, text)
    UI.DrawEx.TextAdditive(
        "UbuntuBold",
        text,
        size,
        x + 1, y + 1, 1, size,
        0, 0, 0, 1.0,
        horJust, 0.5
    )
    UI.DrawEx.TextAdditive(
        "UbuntuBold",
        text,
        size,
        x, y, 1, size,
        color.r, color.g, color.b, 1.0,
        horJust, 0.5
    )
end

function HUD.Create(gameView, player)
    local self = setmetatable({
        gameView        = gameView,
        player          = player,
        icon            = UI.Icon(),

        target          = nil,
        targets         = Systems.CommandView.TrackingList(player, Entity.isTrackable),

        -- TODO : Probably want a reusable prompt thing
        dockPromptAlpha = 0,
        dockable        = nil,
        dockables       = Systems.CommandView.TrackingList(player, Entity.hasDockable),
        aimX            = 0,
        aimY            = 0,
        impacts         = 0,

        children        = List(),
    }, HUD)

    self.icon:setOnDraw(function(ib, focus, active)
        self:onDrawIcon(ib, focus, active)
    end)

    self.targets:update()
    self.dockables:update()

    return self
end

return HUD
