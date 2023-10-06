/// Contains user changes made from outside of the main event loop.
pub enum UserChange {
    CursorVisible(bool),
    CursorPosition(f32, f32),
}
