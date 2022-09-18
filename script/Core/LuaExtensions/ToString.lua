--[[----------------------------------------------------------------------------
  Provides a more-detailed tostring function that automatically retrieves debug
  information for function objects and converts C strings. Note that since
  tostring() is used for print(), using this function to globally replace the
  default tostring will also replace the default print behavior.
----------------------------------------------------------------------------]]--
_tostring = tostring
function tostring (x)
  local result
  if type(x) == 'function' then
    local info = debug.getinfo(x)
    if info.what == 'Lua' then
      if info.linedefined == -1 then
        result = format('[Lua function @ %p %s]', x, info.short_src)
      else
        result = format('[Lua function @ %p -- %s:%d]', x, info.short_src, info.linedefined)
      end
    else
      result = format('[C function @ %p]', x)
    end
  elseif type(x) == 'cdata' then
    if x == nil then
      result = '(nullptr)'
    else
      if ffi.istype('char const*', x) then
        result = format('"%s"', ffi.string(x))
      end
    end
  end
  return result or _tostring(x)
end