pub struct AxisState<const N: usize> {
    current: [f32; N],
    previous: [f32; N],
}

impl<const N: usize> Default for AxisState<N> {
    fn default() -> Self {
        Self {
            current: [0.0; N],
            previous: [0.0; N],
        }
    }
}

impl<const N: usize> AxisState<N> {
    /// Copies current values to previous for delta calculation
    pub fn reset(&mut self) {
        self.previous.copy_from_slice(&self.current);
    }

    pub fn update(&mut self, index: usize, value: f32) -> bool {
        if index < N {
            self.current[index] = value;
            true
        } else {
            false // TODO: wrong index. Return an error/panic?
        }
    }

    /// Returns the current position of the axis
    pub fn value(&self, index: usize) -> f32 {
        if index < N {
            self.current[index]
        } else {
            0.0 // TODO: wrong index. Return an error/panic?
        }
    }

    /// Returns the change in axis position since last reset
    pub fn delta(&self, index: usize) -> f32 {
        if index < N {
            self.current[index] - self.previous[index]
        } else {
            0.0 // TODO: wrong index. Return an error/panic?
        }
    }
}
