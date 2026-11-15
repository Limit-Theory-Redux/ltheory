//! The bendable stroke (`Shape::Stroke`, `ui/stroke.glsl`): one line that
//! bends from straight to a full circle, with its look (core, glow, shade,
//! dark halo) set from Lua.
//!
//! Every stroke is drawn from one internal form, the *wire*: a start point,
//! the direction it leaves in, its length and its curvature (the turn per
//! pixel of length, so `curvature * length` is the whole bend). It is the one
//! form that covers straight lines and full circles alike. Three ways to
//! describe a stroke convert to it, and set up for the same curve they give
//! the same wire:
//! - [`Stroke::between`]: both ends fixed, bent by an angle. Below a full
//!   turn only: at 360 degrees the ends meet and the circle's size is unknown.
//! - [`Stroke::bent`]: start, direction and length fixed ("bend a wire"): it
//!   curls up with its length kept, and 360 degrees closes it into a circle.
//! - [`Stroke::arc`]: centre and radius fixed, from one angle to another.
//!   Can't be straight (that would be a centre infinitely far away).
//!
//! Positions are pixels with y down. Angles are degrees, 0 = up, growing
//! clockwise (like CSS `conic-gradient`); a positive bend turns clockwise.

use glam::Vec2;

use crate::render::{ShaderParam, ShaderParams};

/// The most a stroke bends: a full turn.
pub const STROKE_MAX_BEND: f32 = 360.0;
/// The most `Stroke::between` bends: just short of a full turn, where the
/// circle through both ends would have no size.
pub const STROKE_MAX_BEND_BETWEEN: f32 = 359.0;

/// The number of look parameters (`STROKE_LOOK`).
pub const STROKE_LOOK_LEN: usize = 8;

/// The look of a stroke: the parameters `ui/stroke.glsl` reads from the shape
/// parameter block, by the names Lua sets them with. The defaults draw a
/// plain line in its colour (alpha blended, no glow).
///
/// - `core`, `coreWidth`: strength of the solid core, and how soft its edge
///   is (pixels; the light falls by `e` per `coreWidth` beyond half a pixel
///   past the outline).
/// - `glow`, `glowWidth`: strength and reach (pixels) of the glow around it.
/// - `boost`: multiplies the colour of the light (the engine's other `ui/*`
///   shapes use 2).
/// - `shade`: how much the stroke covers what is behind it: 0 only adds light
///   (neon), 1 blends like normal alpha.
/// - `halo`, `haloWidth`: a dark halo that darkens what is behind the stroke
///   out to `haloWidth` pixels past its outline (0 = none, 1 = black), so a
///   glowing line stays readable on a bright background.
pub const STROKE_LOOK: ShaderParams = ShaderParams::new(&[
    ShaderParam::new("core", 1.0),
    ShaderParam::new("coreWidth", 0.5),
    ShaderParam::new("glow", 0.0),
    ShaderParam::new("glowWidth", 4.0),
    ShaderParam::new("boost", 1.0),
    ShaderParam::new("shade", 1.0),
    ShaderParam::new("halo", 0.0),
    ShaderParam::new("haloWidth", 8.0),
]);

/// Values of the `STROKE_LOOK` parameters, in slot order.
pub type StrokeLook = [f32; STROKE_LOOK_LEN];

/// The largest glow room a stroke's quad gets, in pixels.
const MAX_REACH: f32 = 512.0;

/// How far a look's light and dark halo reach past a stroke's outline, in
/// pixels: where each falls below 1/512 (half a step of 8-bit colour), plus
/// one pixel.
pub fn stroke_look_reach(look: &StrokeLook) -> f32 {
    const VISIBLE: f32 = 512.0;
    let [
        core,
        core_width,
        glow,
        glow_width,
        boost,
        _shade,
        halo,
        halo_width,
    ] = *look;
    // The light is the colour times `boost`; what covers the background is
    // the same curve without it.
    let scale = VISIBLE * boost.abs().max(1.0);

    let mut reach: f32 = 0.0;
    let core_peak = core.abs() * scale;
    if core_peak > 1.0 {
        reach = reach.max(0.5 + core_width.max(0.0) * core_peak.ln());
    }
    let glow_peak = glow.abs() * scale;
    if glow_peak > 1.0 {
        reach = reach.max(glow_width.max(0.0) * glow_peak.ln().powf(4.0 / 3.0));
    }
    if halo > 0.0 {
        reach = reach.max(halo_width.max(0.0));
    }
    (reach + 1.0).min(MAX_REACH)
}

/// Dashes along a stroke: `dash` pixels drawn, then `gap` pixels left out,
/// the pattern shifted back by `phase` pixels (like SVG's
/// `stroke-dashoffset`). Lengths are along the stroke's middle; round caps
/// add half the thickness at both ends of each dash, so a dash of 0 with
/// round caps is a row of dots. Solid when `gap` is 0.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StrokeDash {
    pub dash: f32,
    pub gap: f32,
    pub phase: f32,
}

impl StrokeDash {
    pub fn new(dash: f32, gap: f32, phase: f32) -> Self {
        Self {
            dash: dash.max(0.0),
            gap: gap.max(0.0),
            phase,
        }
    }

    pub fn is_solid(&self) -> bool {
        self.gap <= 0.0
    }
}

/// A fade along a stroke: fully drawn within `visible` pixels of the point
/// `centre` pixels from its start, gone beyond `gone`, smooth in between.
/// With a `period` (a closed circle's length), distances wrap around it.
/// Nothing fades when `gone` is 0.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct StrokeFade {
    pub centre: f32,
    pub visible: f32,
    pub gone: f32,
    pub period: f32,
}

impl StrokeFade {
    /// A fade along an open stroke. `gone` is kept past `visible`, so the
    /// ramp between them is never empty.
    pub fn along(centre: f32, visible: f32, gone: f32) -> Self {
        Self::periodic(centre, visible, gone, 0.0)
    }

    /// A fade whose distances wrap around `period` pixels.
    pub fn periodic(centre: f32, visible: f32, gone: f32, period: f32) -> Self {
        let visible = visible.abs();
        Self {
            centre,
            visible,
            gone: gone.abs().max(visible + 0.01),
            period: period.max(0.0),
        }
    }

    pub fn is_none(&self) -> bool {
        self.gone <= 0.0
    }

    /// As the shader takes it: `[centre, visible, gone, period]`.
    pub fn to_array(&self) -> [f32; 4] {
        [self.centre, self.visible, self.gone, self.period]
    }
}

/// A stroke in its wire form (see the module documentation).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stroke {
    /// Where it starts, in pixels.
    pub start: Vec2,
    /// The unit vector it leaves `start` in.
    pub direction: Vec2,
    /// Its length along its middle, in pixels.
    pub length: f32,
    /// The turn per pixel of length, in radians; positive turns clockwise.
    pub curvature: f32,
    /// Its width across, in pixels. A stroke of thickness 0 is a hairline:
    /// its core is about a pixel wide.
    pub thickness: f32,
    /// Round ends (half circles) instead of flat ones.
    pub round_caps: bool,
    /// Flat ends only: their corners rounded by this many pixels (at most
    /// half the thickness), also on every dash.
    pub corner: f32,
    pub dash: StrokeDash,
    pub fade: StrokeFade,
}

/// The unit vector of a direction in degrees (0 = up, clockwise).
fn heading(degrees: f32) -> Vec2 {
    let (sin, cos) = degrees.to_radians().sin_cos();
    Vec2::new(sin, -cos)
}

/// `v` turned clockwise on screen (y down) by `radians`.
fn rotate(v: Vec2, radians: f32) -> Vec2 {
    let (sin, cos) = radians.sin_cos();
    Vec2::new(v.x * cos - v.y * sin, v.x * sin + v.y * cos)
}

impl Stroke {
    fn wire(start: Vec2, direction: Vec2, length: f32, bend: f32, thickness: f32) -> Self {
        let length = length.max(0.0);
        Self {
            start,
            direction,
            length,
            curvature: if length > 0.0 { bend / length } else { 0.0 },
            thickness: thickness.max(0.0),
            round_caps: false,
            corner: 0.0,
            dash: StrokeDash::default(),
            fade: StrokeFade::default(),
        }
    }

    /// Length fixed ("bend a wire"): from `start`, leaving in `direction`
    /// degrees, `length` pixels long, turning `bend` degrees in all (at most
    /// a full turn either way, which closes it into a circle).
    pub fn bent(start: Vec2, direction: f32, length: f32, bend: f32, thickness: f32) -> Self {
        let bend = bend.clamp(-STROKE_MAX_BEND, STROKE_MAX_BEND).to_radians();
        Self::wire(start, heading(direction), length, bend, thickness)
    }

    /// Endpoints fixed: from `from` to `to`, bent `bend` degrees on the way
    /// (kept below a full turn, `STROKE_MAX_BEND_BETWEEN`). A bend of 0 is the
    /// straight line between them.
    pub fn between(from: Vec2, to: Vec2, bend: f32, thickness: f32) -> Self {
        let chord = to - from;
        let chord_length = chord.length();
        let direction = if chord_length > 0.0 {
            chord / chord_length
        } else {
            Vec2::X
        };
        let bend = bend
            .clamp(-STROKE_MAX_BEND_BETWEEN, STROKE_MAX_BEND_BETWEEN)
            .to_radians();
        let half = 0.5 * bend;
        if half.abs() < 1e-6 {
            return Self::wire(from, direction, chord_length, 0.0, thickness);
        }
        // An arc that turns by `bend` leaves half of it before the chord's
        // direction and arrives half of it after; its length over its chord
        // is the angle over the sine of half of it.
        let length = chord_length * half / half.sin();
        Self::wire(from, rotate(direction, -half), length, bend, thickness)
    }

    /// Centre fixed: around `centre` at `radius` pixels, from `start` degrees
    /// on by `sweep` degrees (negative goes counter-clockwise; at most a full
    /// turn).
    pub fn arc(centre: Vec2, radius: f32, start: f32, sweep: f32, thickness: f32) -> Self {
        let radius = radius.max(0.0);
        let sweep = sweep.clamp(-STROKE_MAX_BEND, STROKE_MAX_BEND);
        // Going clockwise around the centre, the stroke heads a quarter turn
        // on from the direction it sits in.
        let side = if sweep < 0.0 { -1.0 } else { 1.0 };
        let length = radius * sweep.abs().to_radians();
        Self::wire(
            centre + radius * heading(start),
            heading(start + 90.0 * side),
            length,
            sweep.to_radians(),
            thickness,
        )
    }

    /// Nothing to draw: no length.
    pub fn is_empty(&self) -> bool {
        self.length <= 0.0
    }

    /// The whole turn, in radians.
    pub fn bend(&self) -> f32 {
        self.curvature * self.length
    }

    /// The local frame's second axis: `direction` turned a quarter clockwise,
    /// the side a positive curvature turns to.
    pub fn normal(&self) -> Vec2 {
        Vec2::new(-self.direction.y, self.direction.x)
    }

    /// The point `s` pixels along the stroke in its local frame (`start` at
    /// the origin, `direction` along x, `normal` along y).
    pub fn local_point(&self, s: f32) -> Vec2 {
        let k = self.curvature;
        if k.abs() < 1e-9 {
            return Vec2::new(s, 0.0);
        }
        let turn = k * s;
        let half = (0.5 * turn).sin();
        Vec2::new(turn.sin() / k, 2.0 * half * half / k)
    }

    /// From the local frame to the screen.
    pub fn to_screen(&self, local: Vec2) -> Vec2 {
        self.start + local.x * self.direction + local.y * self.normal()
    }

    /// The point `s` pixels along the stroke, on screen.
    pub fn point(&self, s: f32) -> Vec2 {
        self.to_screen(self.local_point(s))
    }

    /// The bounding box of the stroke's middle line in its local frame.
    pub fn local_bounds(&self) -> (Vec2, Vec2) {
        let end = self.local_point(self.length);
        let mut min = Vec2::ZERO.min(end);
        let mut max = Vec2::ZERO.max(end);
        // Where the heading crosses a quarter turn, the line reaches its
        // furthest along x (a quarter, three quarters) or y (a half).
        let turn = self.bend().abs();
        for quarters in 1..=3 {
            let at = quarters as f32 * std::f32::consts::FRAC_PI_2;
            if at < turn {
                let p = self.local_point(at / self.curvature.abs());
                min = min.min(p);
                max = max.max(p);
            }
        }
        (min, max)
    }

    /// The quad to draw: the local bounds grown by half the thickness and
    /// `reach`, as four (screen position, local position) corners in the
    /// order of a quad's triangle fan.
    pub fn quad(&self, reach: f32) -> [(Vec2, Vec2); 4] {
        let (min, max) = self.local_bounds();
        let grow = Vec2::splat(0.5 * self.thickness + reach.max(0.0));
        let (min, max) = (min - grow, max + grow);
        [
            Vec2::new(min.x, min.y),
            Vec2::new(min.x, max.y),
            Vec2::new(max.x, max.y),
            Vec2::new(max.x, min.y),
        ]
        .map(|local| (self.to_screen(local), local))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-3;

    fn assert_close(actual: f32, expected: f32, what: &str) {
        assert!(
            (actual - expected).abs() < EPS,
            "{what}: expected {expected}, got {actual}"
        );
    }

    fn assert_vec(actual: Vec2, expected: Vec2, what: &str) {
        assert!(
            actual.distance(expected) < EPS,
            "{what}: expected {expected}, got {actual}"
        );
    }

    /// The same wire, within float rounding.
    fn assert_same(a: &Stroke, b: &Stroke) {
        assert_vec(a.start, b.start, "start");
        assert_vec(a.direction, b.direction, "direction");
        assert_close(a.length, b.length, "length");
        assert!(
            (a.curvature - b.curvature).abs() < 1e-6,
            "curvature: {} vs {}",
            a.curvature,
            b.curvature
        );
        assert_close(a.thickness, b.thickness, "thickness");
    }

    #[test]
    fn three_descriptions_of_a_quarter_circle_are_one_wire() {
        // Centre (100, 100), radius 50: from the left point, clockwise up to
        // the top point.
        let centre = Vec2::new(100.0, 100.0);
        let arc = Stroke::arc(centre, 50.0, 270.0, 90.0, 4.0);
        let between = Stroke::between(Vec2::new(50.0, 100.0), Vec2::new(100.0, 50.0), 90.0, 4.0);
        let length = 50.0 * std::f32::consts::FRAC_PI_2;
        let bent = Stroke::bent(Vec2::new(50.0, 100.0), 0.0, length, 90.0, 4.0);

        assert_vec(arc.start, Vec2::new(50.0, 100.0), "arc start");
        assert_vec(arc.direction, Vec2::new(0.0, -1.0), "leaves upwards");
        assert_close(arc.length, length, "a quarter of the circumference");
        assert_close(arc.curvature, 1.0 / 50.0, "turns clockwise, radius 50");
        assert_same(&arc, &between);
        assert_same(&arc, &bent);
        assert_vec(
            arc.point(arc.length),
            Vec2::new(100.0, 50.0),
            "ends at the top",
        );
    }

    #[test]
    fn counter_clockwise_bends_are_one_wire() {
        // From the top point counter-clockwise down to the left point.
        let centre = Vec2::new(0.0, 0.0);
        let arc = Stroke::arc(centre, 20.0, 0.0, -90.0, 1.0);
        let between = Stroke::between(Vec2::new(0.0, -20.0), Vec2::new(-20.0, 0.0), -90.0, 1.0);
        let length = 20.0 * std::f32::consts::FRAC_PI_2;
        let bent = Stroke::bent(Vec2::new(0.0, -20.0), 270.0, length, -90.0, 1.0);

        assert_vec(arc.direction, Vec2::new(-1.0, 0.0), "leaves to the left");
        assert!(arc.curvature < 0.0);
        assert_same(&arc, &between);
        assert_same(&arc, &bent);
        assert_vec(arc.point(arc.length), Vec2::new(-20.0, 0.0), "ends left");
    }

    #[test]
    fn half_circles_and_wide_bends_are_one_wire() {
        // From the left point clockwise over the top to the right point.
        let centre = Vec2::new(10.0, 10.0);
        let arc = Stroke::arc(centre, 30.0, 270.0, 180.0, 2.0);
        let between = Stroke::between(Vec2::new(-20.0, 10.0), Vec2::new(40.0, 10.0), 180.0, 2.0);
        assert_same(&arc, &between);
        assert_vec(
            arc.point(0.5 * arc.length),
            Vec2::new(10.0, -20.0),
            "over the top",
        );

        // Three quarters: the chord is shorter than the diameter.
        let arc = Stroke::arc(centre, 30.0, 0.0, 270.0, 2.0);
        let between = Stroke::between(arc.start, arc.point(arc.length), 270.0, 2.0);
        assert_same(&arc, &between);
        assert_vec(arc.point(arc.length), Vec2::new(-20.0, 10.0), "ends left");
    }

    #[test]
    fn straight_strokes_are_one_wire() {
        let from = Vec2::new(10.0, 20.0);
        let to = Vec2::new(40.0, 60.0);
        let between = Stroke::between(from, to, 0.0, 3.0);
        // From (10, 20) to (40, 60) is (0.6, 0.8) per pixel: sin = 0.6 and
        // -cos = 0.8, about 143 degrees.
        let direction = 0.6f32.atan2(-0.8).to_degrees();
        let bent = Stroke::bent(from, direction, 50.0, 0.0, 3.0);

        assert_eq!(between.curvature, 0.0);
        assert_close(between.length, 50.0, "length");
        assert_vec(between.direction, Vec2::new(0.6, 0.8), "direction");
        assert_same(&between, &bent);
        assert_vec(between.point(50.0), to, "ends at `to`");
    }

    #[test]
    fn a_full_turn_closes_the_circle() {
        let centre = Vec2::new(0.0, 0.0);
        let arc = Stroke::arc(centre, 40.0, 270.0, 360.0, 2.0);
        let length = 40.0 * std::f32::consts::TAU;
        let bent = Stroke::bent(Vec2::new(-40.0, 0.0), 0.0, length, 360.0, 2.0);
        assert_same(&arc, &bent);
        assert_close(arc.bend(), std::f32::consts::TAU, "a full turn");
        assert_vec(arc.point(arc.length), arc.start, "ends where it starts");

        // More than a full turn is a full turn.
        assert_same(&Stroke::bent(bent.start, 0.0, length, 720.0, 2.0), &bent);
        assert_same(&Stroke::arc(centre, 40.0, 270.0, 400.0, 2.0), &arc);
    }

    #[test]
    fn between_stays_below_a_full_turn() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 0.0);
        let stroke = Stroke::between(a, b, 360.0, 1.0);
        assert_close(
            stroke.bend().to_degrees(),
            STROKE_MAX_BEND_BETWEEN,
            "bend clamped",
        );
        assert!(stroke.length.is_finite());
        assert_vec(stroke.point(stroke.length), b, "still ends at `to`");
    }

    #[test]
    fn nothing_to_draw_without_length() {
        assert!(Stroke::between(Vec2::ONE, Vec2::ONE, 90.0, 2.0).is_empty());
        assert!(Stroke::bent(Vec2::ONE, 0.0, 0.0, 90.0, 2.0).is_empty());
        assert!(Stroke::arc(Vec2::ONE, 10.0, 0.0, 0.0, 2.0).is_empty());
        assert!(Stroke::arc(Vec2::ONE, 0.0, 0.0, 90.0, 2.0).is_empty());
        assert!(!Stroke::bent(Vec2::ONE, 0.0, 1.0, 0.0, 0.0).is_empty());
    }

    #[test]
    fn local_bounds_hold_the_whole_curve() {
        let r = 10.0;
        let quarter = Stroke::bent(Vec2::ZERO, 90.0, r * std::f32::consts::FRAC_PI_2, 90.0, 0.0);
        let (min, max) = quarter.local_bounds();
        assert_vec(min, Vec2::new(0.0, 0.0), "quarter min");
        assert_vec(max, Vec2::new(r, r), "quarter max");

        let half = Stroke::bent(Vec2::ZERO, 90.0, r * std::f32::consts::PI, 180.0, 0.0);
        let (min, max) = half.local_bounds();
        assert_vec(min, Vec2::new(0.0, 0.0), "half min");
        assert_vec(max, Vec2::new(r, 2.0 * r), "half max");

        let full = Stroke::bent(Vec2::ZERO, 90.0, r * std::f32::consts::TAU, 360.0, 0.0);
        let (min, max) = full.local_bounds();
        assert_vec(min, Vec2::new(-r, 0.0), "full min");
        assert_vec(max, Vec2::new(r, 2.0 * r), "full max");

        // Turning the other way mirrors it across the x axis.
        let mirrored = Stroke::bent(Vec2::ZERO, 90.0, r * std::f32::consts::PI, -180.0, 0.0);
        let (min, max) = mirrored.local_bounds();
        assert_vec(min, Vec2::new(0.0, -2.0 * r), "mirrored min");
        assert_vec(max, Vec2::new(r, 0.0), "mirrored max");

        let straight = Stroke::bent(Vec2::ZERO, 90.0, 25.0, 0.0, 0.0);
        let (min, max) = straight.local_bounds();
        assert_vec(min, Vec2::ZERO, "straight min");
        assert_vec(max, Vec2::new(25.0, 0.0), "straight max");
    }

    #[test]
    fn the_quad_holds_the_stroke_and_its_reach() {
        // A horizontal line from (10, 20), 30 px long, 4 px thick, 5 px reach.
        let stroke = Stroke::bent(Vec2::new(10.0, 20.0), 90.0, 30.0, 0.0, 4.0);
        let quad = stroke.quad(5.0);
        let screen: Vec<Vec2> = quad.iter().map(|c| c.0).collect();
        let local: Vec<Vec2> = quad.iter().map(|c| c.1).collect();
        assert_vec(local[0], Vec2::new(-7.0, -7.0), "local min");
        assert_vec(local[2], Vec2::new(37.0, 7.0), "local max");
        assert_vec(screen[0], Vec2::new(3.0, 13.0), "screen top left");
        assert_vec(screen[2], Vec2::new(47.0, 27.0), "screen bottom right");

        // Rotated: a line going down keeps local x along it.
        let down = Stroke::bent(Vec2::new(0.0, 0.0), 180.0, 10.0, 0.0, 0.0);
        let quad = down.quad(1.0);
        assert_vec(quad[2].0, Vec2::new(-1.0, 11.0), "local max on screen");
    }

    #[test]
    fn fades_keep_a_ramp() {
        let fade = StrokeFade::along(50.0, 20.0, 10.0);
        assert!(fade.gone > fade.visible);
        assert!(!fade.is_none());
        assert!(StrokeFade::default().is_none());
        assert_eq!(
            StrokeFade::periodic(1.0, 2.0, 3.0, 4.0).to_array(),
            [1.0, 2.0, 3.0, 4.0]
        );
        assert!(StrokeDash::default().is_solid());
        assert!(!StrokeDash::new(0.0, 4.0, 0.0).is_solid());
    }

    #[test]
    fn looks_reach_as_far_as_their_glow_and_halo() {
        let plain: StrokeLook = STROKE_LOOK.defaults();
        // The default core: half a pixel plus a soft edge, a few pixels.
        let reach = stroke_look_reach(&plain);
        assert!(reach > 3.0 && reach < 6.0, "{reach}");

        // The engine's wedge glow (0.4, falloff 0.3 per px, colour doubled)
        // fades out at about 32 px, the room `DrawEx.Wedge` gives it.
        let mut wedge = plain;
        wedge[2] = 0.4;
        wedge[3] = 1.0 / 0.3;
        wedge[4] = 2.0;
        let reach = stroke_look_reach(&wedge);
        assert!(reach > 30.0 && reach < 40.0, "{reach}");

        // A dark halo reaches its width.
        let mut halo = plain;
        halo[6] = 0.7;
        halo[7] = 60.0;
        assert_close(stroke_look_reach(&halo), 61.0, "halo reach");

        // Absurd values stay bounded.
        let mut huge = plain;
        huge[3] = 1e6;
        huge[2] = 1.0;
        assert_eq!(stroke_look_reach(&huge), MAX_REACH);
    }

    #[test]
    fn the_shader_reads_the_look_in_slot_order() {
        // `ui/stroke.glsl` names each look parameter with a define on its
        // slot: parameter `i` is component `i % 4` of `look{i / 4}`.
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../res/shader/fragment/ui/stroke.glsl");
        let src = std::fs::read_to_string(&path).expect("read stroke.glsl");
        for (i, param) in STROKE_LOOK.params.iter().enumerate() {
            let component = ["x", "y", "z", "w"][i % 4];
            let define = format!("#define {} (look{}.{component})", param.name, i / 4);
            assert!(
                src.lines().any(|l| l.trim_end() == define),
                "stroke.glsl must say `{define}`"
            );
        }
    }
}
