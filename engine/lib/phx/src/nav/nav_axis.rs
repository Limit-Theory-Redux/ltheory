use ::nav::Axis;

// The values come from `nav::Axis`, so the buffer layout has one source.
const PITCH: u32 = Axis::Pitch as u32;
const YAW: u32 = Axis::Yaw as u32;
const ROLL: u32 = Axis::Roll as u32;
const THRUST_FORWARD: u32 = Axis::ThrustForward as u32;
const THRUST_RIGHT: u32 = Axis::ThrustRight as u32;
const THRUST_UP: u32 = Axis::ThrustUp as u32;

/// A control axis: the position of its value in an agent's slice of the buffer that
/// `NavWorld::copy_axes` fills. Every value is in `[-1, 1]`.
#[luajit_ffi_gen::luajit_ffi(repr = "u32")]
#[derive(Debug, Clone, Copy)]
pub enum NavAxis {
    Pitch = PITCH,
    Yaw = YAW,
    Roll = ROLL,
    ThrustForward = THRUST_FORWARD,
    ThrustRight = THRUST_RIGHT,
    ThrustUp = THRUST_UP,
}
