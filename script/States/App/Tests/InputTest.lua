local InputTest = require('States.Application')

<<<<<<< HEAD
function InputTest:onInit()
    --InputBindings.Register(UI.Bindings.Keyboard)
    --InputBindings.Init();
end

function InputTest:onUpdate(dt)
    --[[ NOTE : Low Level API Usage Style 1 - Direct State Queries
=======
function InputTest:onInit ()
  --InputBindings.Register(UI.Bindings.Keyboard)
  --InputBindings.Init();
end

function InputTest:onUpdate (dt)
  --[[ NOTE : Low Level API Usage Style 1 - Direct State Queries
>>>>>>> 1b58bb0278295d31845972084d1313877cd21e29
  for i = 1, 512 do
    if Input.GetPressed(i)  then printf('Pressed  - %s', ffi.string(libphx.Button_ToString(i))) end
    if Input.GetReleased(i) then printf('Released - %s', ffi.string(libphx.Button_ToString(i))) end
  end
  --]]

<<<<<<< HEAD
    ---[[ NOTE : Low Level API Usage Style 2 - Event Queue
    local self = InputTest
    self.eventCount = Input.GetEventCount()
    for i = 1, Input.GetEventCount() do
        local event = InputEvent()
        Input.GetNextEvent(event)
        if event.deviceType == DeviceType.Gamepad and
            (Bit.Has32(event.state, State.Pressed) or
                Bit.Has32(event.state, State.Released))
        then
            print(event)
        end
    end
    --]]

    --[[ NOTE : High Level API Usage Style 1 - Direct State Queries
=======
  ---[[ NOTE : Low Level API Usage Style 2 - Event Queue
  local self = InputTest
  self.eventCount = Input.GetEventCount()
  for i = 1, Input.GetEventCount() do
    local event = InputEvent()
    Input.GetNextEvent(event)
    if event.deviceType == DeviceType.Gamepad and
       (Bit.Has32(event.state, State.Pressed) or
        Bit.Has32(event.state, State.Released))
    then
      print(event)
    end
  end
  --]]

  --[[ NOTE : High Level API Usage Style 1 - Direct State Queries
>>>>>>> 1b58bb0278295d31845972084d1313877cd21e29
  InputBindings.Update();
  --]]


<<<<<<< HEAD
    --[[ NOTE : High Level API Usage Style 2 - Event Stream --]]
    --[[ NOTE : High Level API Usage Style 3 - Callbacks --]]
end

function InputTest:onDraw()
end

function InputTest:onExit()
    --InputBindings.Unregister(UI.Bindings.Keyboard)
    --InputBindings.Free();
=======
  --[[ NOTE : High Level API Usage Style 2 - Event Stream --]]
  --[[ NOTE : High Level API Usage Style 3 - Callbacks --]]
end

function InputTest:onDraw ()
end

function InputTest:onExit ()
  --InputBindings.Unregister(UI.Bindings.Keyboard)
  --InputBindings.Free();
>>>>>>> 1b58bb0278295d31845972084d1313877cd21e29
end

return InputTest
