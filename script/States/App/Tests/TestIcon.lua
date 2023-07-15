local Test = require('States.Application')
local Icon = require('UI.Icon')
local IconButton = require('UI.IconButton')
local rng = RNG.FromTime()
local icon

<<<<<<< HEAD
function Test:onInit()
    Cache.Texture('icon/lbumper', true)
    --  icon = IconButton.Create('icon/lbumper', nil)

    icon = Tex2D.Load("./res/images/LTR_logo2.png") -- load the LTR logo

    UI.DrawEx.Icon(icon, 100, 100, 100, 100, Color(1, 1, 1, 1))


    --  icon:addPoint(0.5, 0.5)
    --  icon:addPoint(0.2, 0.2)
end

function Test:onInput() end

function Test:onUpdate(dt) end

function Test:onDraw()
    Draw.Clear(0.1, 0.1, 0.1, 1.0)
    BlendMode.PushAdditive()
    local y = 16
    local size = 16
    for i = 1, 6 do
        icon:draw(16, y, size, 0.1, 0.5, 1.0, 1.0)
        y = y + size + 4
        size = size * 2
    end
    BlendMode.Pop()
=======
function Test:onInit ()
  Cache.Texture('icon/lbumper', true)
--  icon = IconButton.Create('icon/lbumper', nil)

  icon = Tex2D.Load("./res/images/LTR_logo2.png") -- load the LTR logo

  UI.DrawEx.Icon(icon, 100, 100, 100, 100, Color(1,1,1,1))


--  icon:addPoint(0.5, 0.5)
--  icon:addPoint(0.2, 0.2)
end

function Test:onInput () end
function Test:onUpdate (dt) end

function Test:onDraw ()
  Draw.Clear(0.1, 0.1, 0.1, 1.0)
  BlendMode.PushAdditive()
  local y = 16
  local size = 16
  for i = 1, 6 do
    icon:draw(16, y, size, 0.1, 0.5, 1.0, 1.0)
    y = y + size + 4
    size = size * 2
  end
  BlendMode.Pop()
>>>>>>> 1b58bb0278295d31845972084d1313877cd21e29
end

return Test
