pub struct AxisState<const N: usize> {
    axes: [f32; N],
}

impl<const N: usize> Default for AxisState<N> {
    fn default() -> Self {
        Self { axes: [0.0; N] }
    }
}

impl<const N: usize> AxisState<N> {
    pub fn reset(&mut self) {
        self.axes.fill(0.0);
    }

    pub fn update(&mut self, index: usize, value: f32) -> bool {
        if index < N {
            self.axes[index] = value;
            true
        } else {
            false
        }
    }

    pub fn value(&self, index: usize) -> f32 {
        if index < N {
            self.axes[index]
        } else {
            0.0 // TODO: wrong index. Return an error/panic?
        }
    }
}
