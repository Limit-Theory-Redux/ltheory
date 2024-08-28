use mlua::{FromLua, IntoLua, Lua, Result, Value};

use super::Payload;

impl IntoLua<'_> for Payload {
    fn into_lua(self, _lua: &'_ Lua) -> Result<Value<'_>> {
        todo!()
    }
}

impl FromLua<'_> for Payload {
    fn from_lua(_value: Value<'_>, _lua: &'_ Lua) -> Result<Self> {
        todo!()
    }
}
