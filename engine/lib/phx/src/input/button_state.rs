pub struct ButtonState<const N: usize> {
    transitions: [u32; N],
    buttons: [bool; N],
}

impl<const N: usize> Default for ButtonState<N> {
    fn default() -> Self {
        Self {
            transitions: [0; N],
            buttons: [false; N],
        }
    }
}

impl<const N: usize> ButtonState<N> {
    pub fn reset(&mut self) {
        self.transitions.fill(0);
    }

    pub fn update(&mut self, index: usize, pressed: bool) -> bool {
        if index < N {
            self.transitions[index] += 1;
            self.buttons[index] = pressed;

            true
        } else {
            true // TODO: wrong index. Return an error/panic?
        }
    }

    pub fn is_pressed(&self, index: usize) -> bool {
        if index < N {
            if self.buttons[index] {
                self.transitions[index] > 0
            } else {
                self.transitions[index] > 1
            }
        } else {
            false // TODO: wrong index. Return an error/panic?
        }
    }

    pub fn is_down(&self, index: usize) -> bool {
        if index < N {
            self.buttons[index] || self.transitions[index] > 0
        } else {
            false // TODO: wrong index. Return an error/panic?
        }
    }

    pub fn is_released(&self, index: usize) -> bool {
        if index < N {
            if self.buttons[index] {
                self.transitions[index] > 1
            } else {
                self.transitions[index] > 0
            }
        } else {
            false // TODO: wrong index. Return an error/panic?
        }
    }
}
