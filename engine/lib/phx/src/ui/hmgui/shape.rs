use glam::Vec2;

use super::{HmGuiLookId, HmGuiLookOverrides, HmGuiLooks};
use crate::render::{
    Color, STROKE_LOOK, Shape, Stroke, StrokeDash, StrokeFade, StrokeLook, UIRenderer,
    UIShapeGeometry,
};

/// What a shape widget draws.
///
/// The round kinds (annulus, circle, point, ring, wedge, and the arc stroke)
/// are centred in their widget and as wide as its smaller side: the widget's
/// size is the shape's size. Triangles, lines and the other strokes place
/// their points at fractions of the widget (0, 0 = top left, 1, 1 = bottom
/// right), like CSS `clip-path: polygon()`. Glow reaches past the widget's
/// edge (like a CSS `box-shadow`).
///
/// Annulus, ring, wedge, line and the strokes are all drawn as one bendable
/// stroke (`render::Stroke`), so they take a look, dashes, caps and a fade
/// along their length; circle, point and triangle keep their own shaders.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HmGuiShapeKind {
    /// A band between two circles, `thickness` pixels wide, from the
    /// widget's edge inwards.
    Annulus { thickness: f32 },
    /// A filled circle with a soft edge.
    Circle,
    /// A point of light: a small bright core whose soft glow fills the
    /// widget (the only kind that stays inside it).
    Point,
    /// A thin circle line.
    Ring,
    /// A piece of a band: `thickness` pixels deep from the widget's edge
    /// inwards, from `start` to `start + sweep` degrees, where 0 is up and
    /// angles grow clockwise (like CSS `conic-gradient`).
    Wedge {
        thickness: f32,
        start: f32,
        sweep: f32,
    },
    /// A filled triangle with soft edges.
    Triangle { points: [Vec2; 3] },
    /// A thin line.
    Line { from: Vec2, to: Vec2 },
    /// A bendable stroke, described one of three ways.
    Stroke(HmGuiStrokeForm),
}

/// The three ways to describe a stroke widget (see `render::Stroke`), each
/// with its `thickness` in pixels. Points are fractions of the widget, like a
/// triangle's; lengths are pixels; angles are degrees (0 = up, clockwise), and
/// a positive bend turns clockwise.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HmGuiStrokeForm {
    /// Endpoints fixed: from `from` to `to`, bent `bend` degrees on the way
    /// (below a full turn).
    Between {
        from: Vec2,
        to: Vec2,
        bend: f32,
        thickness: f32,
    },
    /// Length fixed ("bend a wire"): from `start`, leaving in `direction`,
    /// `length` pixels long, turning `bend` degrees in all (a full turn
    /// closes it into a circle).
    Bent {
        start: Vec2,
        direction: f32,
        length: f32,
        bend: f32,
        thickness: f32,
    },
    /// Centre fixed: around the widget's centre, its middle line on the
    /// circle as wide as the widget's smaller side, from `start` on by
    /// `sweep` degrees.
    Arc {
        start: f32,
        sweep: f32,
        thickness: f32,
    },
}

impl HmGuiShapeKind {
    /// Centred in the widget, so it can take a fade window.
    pub fn is_round(&self) -> bool {
        match self {
            Self::Triangle { .. } | Self::Line { .. } => false,
            Self::Stroke(form) => matches!(form, HmGuiStrokeForm::Arc { .. }),
            _ => true,
        }
    }

    /// Drawn as a stroke, so it takes a look, dashes, caps and a fade along
    /// its length.
    pub fn is_stroke(&self) -> bool {
        matches!(
            self,
            Self::Annulus { .. }
                | Self::Ring
                | Self::Wedge { .. }
                | Self::Line { .. }
                | Self::Stroke(_)
        )
    }
}

/// The fade window of a round shape, in degrees (0 = up, clockwise): fully
/// drawn within `visible` of the direction `centre`, gone beyond `gone`,
/// smooth in between. Measured from the shape's centre, so shapes that share
/// a centre fade as one (the pieces of a curved tape).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HmGuiShapeFade {
    pub centre: f32,
    pub visible: f32,
    pub gone: f32,
}

/// How a stroke is drawn beyond its shape (`Gui:setStroke*`).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct HmGuiStrokeStyle {
    /// A named look; `None` draws with the kind's own (`stroke`, or for the
    /// annulus, ring, wedge and line their built-in look for `glow`).
    pub look: Option<HmGuiLookId>,
    /// Single look values of this widget, on top of the look.
    pub overrides: HmGuiLookOverrides,
    pub dash: StrokeDash,
    /// `None`: the kind's own (round for a line, flat for the others).
    pub round_caps: Option<bool>,
    /// How much flat ends' corners round, in pixels. `None`: the kind's own
    /// (a wedge's 2, sharp for the others).
    pub corner: Option<f32>,
    /// A fade along the length, in pixels from the start. Takes the place of
    /// the angle window (`HmGuiShape::fade`).
    pub fade: Option<StrokeFade>,
}

/// A shape widget: one of the engine's `ui/*` shapes or a bendable stroke in
/// the layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HmGuiShape {
    pub kind: HmGuiShapeKind,
    pub color: Color,
    /// The shape calls from before strokes: draw additive (brightens what is
    /// behind it; neon) instead of alpha blended (can also darken it). For
    /// the strokes the look decides.
    pub glow: bool,
    /// Round shapes only.
    pub fade: Option<HmGuiShapeFade>,
    /// Strokes only.
    pub stroke: HmGuiStrokeStyle,
    /// Screen pixels per design pixel (`HmGui::set_scale`): the stroke's
    /// lengths (thickness, length, corners, dashes, fade along it) and its
    /// look's widths are multiplied by it when it is drawn.
    pub scale: f32,
}

/// Room for the glow of `triangle.glsl` (as `DrawEx.Tri`).
const PAD_TRIANGLE: f32 = 32.0;

/// The rounded corners of a wedge's ends (the bevel of `wedge.glsl`).
const WEDGE_CORNER: f32 = 2.0;

/// How far inside each edge of an annulus' band the stroke's outline sits:
/// the stroke's core reaches half a pixel past its outline and then softens,
/// so this keeps the band's half-bright edge where `annulus.glsl` had it.
const ANNULUS_EDGE: f32 = 1.0;

impl HmGuiShape {
    /// A shape with nothing but its kind, colour and blend set.
    pub fn new(kind: HmGuiShapeKind, color: Color, glow: bool) -> Self {
        Self {
            kind,
            color,
            glow,
            fade: None,
            stroke: HmGuiStrokeStyle::default(),
            scale: 1.0,
        }
    }

    /// The engine shape and where to draw it for a widget at `pos` with
    /// `size`, or `None` when there is nothing to draw (an empty widget, no
    /// thickness, no sweep, a line or triangle without length or area).
    pub fn geometry(
        &self,
        pos: Vec2,
        size: Vec2,
        looks: &HmGuiLooks,
    ) -> Option<(Shape, UIShapeGeometry)> {
        match self.kind {
            HmGuiShapeKind::Triangle { points } => {
                let points = points.map(|p| pos + p * size);
                let area = (points[1] - points[0]).perp_dot(points[2] - points[0]);
                if area.abs() < 1e-3 {
                    return None;
                }
                let shape = if self.glow {
                    Shape::Triangle
                } else {
                    Shape::TriangleAlpha
                };
                let pad = PAD_TRIANGLE;
                Some((shape, UIShapeGeometry::Triangle { points, pad }))
            }
            HmGuiShapeKind::Circle | HmGuiShapeKind::Point => self.round_geometry(pos, size),
            _ => {
                let (stroke, own_look) = self.to_stroke(pos, size)?;
                let look = self.stroke.look.unwrap_or(own_look);
                let mut look = looks.resolve(look, &self.stroke.overrides);
                scale_look_widths(&mut look, self.scale);
                Some((Shape::Stroke, UIShapeGeometry::Stroke { stroke, look }))
            }
        }
    }

    /// The circle and point: a square quad centred in the widget.
    fn round_geometry(&self, pos: Vec2, size: Vec2) -> Option<(Shape, UIShapeGeometry)> {
        let radius = 0.5 * size.x.min(size.y);
        if radius <= 0.0 {
            return None;
        }

        let (shape, pad, params) = match self.kind {
            HmGuiShapeKind::Circle => {
                // `circle.glsl` softens the edge over 1/32 of the quad, so its
                // room grows with the circle.
                let pad = 0.2 * radius + 2.0;
                let shape = if self.glow {
                    Shape::Circle
                } else {
                    Shape::CircleAlpha
                };
                (shape, pad, [radius, 0.0, 0.0, 0.0])
            }
            HmGuiShapeKind::Point => {
                let shape = if self.glow {
                    Shape::PointGlow
                } else {
                    Shape::Point
                };
                (shape, 0.0, [0.0; 4])
            }
            _ => unreachable!("not a circle or point"),
        };

        // A square quad keeps the round shaders round in a widget of any
        // aspect ratio, and its centre is the shape's (the fade window's).
        let half = radius + pad;
        let center = pos + 0.5 * size;

        Some((
            shape,
            UIShapeGeometry::Quad {
                pos: center - half,
                size: Vec2::splat(2.0 * half),
                params,
                fade: self.fade.map(|f| fade_window(&f)).unwrap_or([0.0; 4]),
            },
        ))
    }

    /// The stroke a stroke kind draws in a widget at `pos` with `size`, with
    /// its style applied, and the look it draws with unless it names one.
    pub fn to_stroke(&self, pos: Vec2, size: Vec2) -> Option<(Stroke, HmGuiLookId)> {
        let centre = pos + 0.5 * size;
        let radius = 0.5 * size.x.min(size.y);
        let at = |p: Vec2| pos + p * size;
        let pick = |alpha, glow| if self.glow { glow } else { alpha };
        let px = |length: f32| length * self.scale; // design to screen pixels

        // The stroke, its own look, whether its caps are round, and for the
        // ones around the centre the arc (radius, start, sweep) a fade window
        // is measured on.
        let (mut stroke, look, round, arc) = match self.kind {
            HmGuiShapeKind::Annulus { thickness } => {
                let thickness = px(thickness);
                if thickness <= 0.0 || radius <= 0.0 {
                    return None;
                }
                let inner = (radius - thickness).max(0.0);
                let middle = 0.5 * (inner + radius);
                let width = (radius - inner - 2.0 * ANNULUS_EDGE).max(0.0);
                let look = pick(HmGuiLookId::ANNULUS, HmGuiLookId::ANNULUS_GLOW);
                let stroke = Stroke::arc(centre, middle, 0.0, 360.0, width);
                (stroke, look, false, Some((middle, 0.0, 360.0)))
            }
            HmGuiShapeKind::Ring => {
                if radius <= 0.0 {
                    return None;
                }
                let look = pick(HmGuiLookId::RING, HmGuiLookId::RING_GLOW);
                let stroke = Stroke::arc(centre, radius, 0.0, 360.0, 0.0);
                (stroke, look, false, Some((radius, 0.0, 360.0)))
            }
            HmGuiShapeKind::Wedge {
                thickness,
                start,
                sweep,
            } => {
                let thickness = px(thickness);
                if thickness <= 0.0 || sweep == 0.0 || radius <= 0.0 {
                    return None;
                }
                let inner = (radius - thickness).max(0.0);
                let middle = 0.5 * (inner + radius);
                let look = pick(HmGuiLookId::WEDGE, HmGuiLookId::WEDGE_GLOW);
                let stroke = Stroke::arc(centre, middle, start, sweep, radius - inner);
                (stroke, look, false, Some((middle, start, sweep)))
            }
            HmGuiShapeKind::Line { from, to } => {
                let look = pick(HmGuiLookId::LINE, HmGuiLookId::LINE_GLOW);
                (
                    Stroke::between(at(from), at(to), 0.0, 0.0),
                    look,
                    true,
                    None,
                )
            }
            HmGuiShapeKind::Stroke(form) => match form {
                HmGuiStrokeForm::Between {
                    from,
                    to,
                    bend,
                    thickness,
                } => {
                    let stroke = Stroke::between(at(from), at(to), bend, px(thickness));
                    (stroke, HmGuiLookId::STROKE, false, None)
                }
                HmGuiStrokeForm::Bent {
                    start,
                    direction,
                    length,
                    bend,
                    thickness,
                } => {
                    let stroke =
                        Stroke::bent(at(start), direction, px(length), bend, px(thickness));
                    (stroke, HmGuiLookId::STROKE, false, None)
                }
                HmGuiStrokeForm::Arc {
                    start,
                    sweep,
                    thickness,
                } => {
                    if radius <= 0.0 {
                        return None;
                    }
                    let stroke = Stroke::arc(centre, radius, start, sweep, px(thickness));
                    (
                        stroke,
                        HmGuiLookId::STROKE,
                        false,
                        Some((radius, start, sweep)),
                    )
                }
            },
            HmGuiShapeKind::Circle | HmGuiShapeKind::Point | HmGuiShapeKind::Triangle { .. } => {
                return None;
            }
        };
        if stroke.is_empty() {
            return None;
        }

        stroke.round_caps = self.stroke.round_caps.unwrap_or(round);
        let corner = if matches!(self.kind, HmGuiShapeKind::Wedge { .. }) {
            WEDGE_CORNER
        } else {
            0.0
        };
        stroke.corner = px(self.stroke.corner.unwrap_or(corner));
        let dash = self.stroke.dash;
        stroke.dash = StrokeDash {
            dash: px(dash.dash),
            gap: px(dash.gap),
            phase: px(dash.phase),
        };
        stroke.fade = match (self.stroke.fade, self.fade, arc) {
            (Some(fade), ..) => StrokeFade {
                centre: px(fade.centre),
                visible: px(fade.visible),
                gone: px(fade.gone),
                period: px(fade.period),
            },
            (None, Some(window), Some((radius, start, sweep))) => {
                arc_fade(&window, radius, start, sweep)
            }
            _ => StrokeFade::default(),
        };
        Some((stroke, look))
    }

    pub fn draw(&self, looks: &HmGuiLooks, renderer: &mut UIRenderer, pos: Vec2, size: Vec2) {
        if let Some((shape, geometry)) = self.geometry(pos, size, looks) {
            renderer.shape(shape, geometry, self.color);
        }
    }

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = crate::ui::hmgui::IDENT.repeat(ident).to_string();

        println!("{ident_str}- kind:   {:?}", self.kind);
        println!("{ident_str}- color:  {:?}", self.color);
        println!("{ident_str}- glow:   {}", self.glow);
        println!("{ident_str}- fade:   {:?}", self.fade);
        if self.kind.is_stroke() {
            println!("{ident_str}- stroke: {:?}", self.stroke);
        }
    }
}

/// Multiply a look's widths (`coreWidth`, `glowWidth`, `haloWidth`, in
/// pixels) by a widget's scale.
fn scale_look_widths(look: &mut StrokeLook, scale: f32) {
    if scale == 1.0 {
        return;
    }
    for name in ["coreWidth", "glowWidth", "haloWidth"] {
        if let Some(slot) = STROKE_LOOK.slot(name) {
            look[slot] *= scale;
        }
    }
}

/// A fade window in degrees (0 = up, clockwise) as the circle and point
/// shaders take it (`Renderer::imm_shape_faded`): `[centre, visible, gone, 1]`
/// in radians, 0 = right, counter-clockwise. `gone` is kept past `visible`,
/// so the ramp between them is never empty.
pub fn fade_window(fade: &HmGuiShapeFade) -> [f32; 4] {
    let visible = fade.visible.abs();
    let gone = fade.gone.abs().max(visible + 0.01);
    [
        (90.0 - fade.centre)
            .to_radians()
            .rem_euclid(std::f32::consts::TAU),
        visible.to_radians(),
        gone.to_radians(),
        1.0,
    ]
}

/// A fade window as a fade along an arc of `radius` pixels from `start` on by
/// `sweep` degrees. Along an arc, the angle around its centre and the
/// distance from its start go together, so this fades every pixel exactly
/// as the window measured from the centre would; the distances wrap around
/// the full circle.
pub fn arc_fade(window: &HmGuiShapeFade, radius: f32, start: f32, sweep: f32) -> StrokeFade {
    let side = if sweep < 0.0 { -1.0 } else { 1.0 };
    let to_px = |degrees: f32| radius * degrees.to_radians();
    let period = radius * std::f32::consts::TAU;
    let centre = to_px(side * (window.centre - start));
    StrokeFade::periodic(
        if period > 0.0 {
            centre.rem_euclid(period)
        } else {
            centre
        },
        to_px(window.visible),
        to_px(window.gone),
        period,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-3,
            "expected {expected}, got {actual}"
        );
    }

    fn assert_vec(actual: Vec2, expected: Vec2) {
        assert!(
            actual.distance(expected) < 1e-3,
            "expected {expected}, got {actual}"
        );
    }

    fn make(kind: HmGuiShapeKind, glow: bool) -> HmGuiShape {
        HmGuiShape::new(kind, Color::WHITE, glow)
    }

    #[test]
    fn scaled_strokes_draw_their_lengths_bigger() {
        let mut wedge = make(
            HmGuiShapeKind::Wedge {
                thickness: 10.0,
                start: 0.0,
                sweep: 90.0,
            },
            false,
        );
        wedge.stroke.dash = StrokeDash::new(4.0, 2.0, 1.0);
        wedge.scale = 2.0;

        let (stroke, _) = wedge.to_stroke(Vec2::ZERO, Vec2::splat(100.0)).unwrap();
        assert_close(stroke.thickness, 20.0);
        assert_close(stroke.corner, 2.0 * WEDGE_CORNER);
        assert_close(stroke.dash.dash, 8.0);
        assert_close(stroke.dash.gap, 4.0);
        assert_close(stroke.dash.phase, 2.0);
    }

    #[test]
    fn scaled_strokes_widen_their_look() {
        let looks = HmGuiLooks::new();
        let mut line = make(
            HmGuiShapeKind::Line {
                from: Vec2::ZERO,
                to: Vec2::ONE,
            },
            false,
        );
        let look_of = |line: &HmGuiShape| match line.geometry(Vec2::ZERO, Vec2::splat(50.0), &looks)
        {
            Some((_, UIShapeGeometry::Stroke { look, .. })) => look,
            _ => panic!("a line draws as a stroke"),
        };

        let plain = look_of(&line);
        line.scale = 3.0;
        let scaled = look_of(&line);

        for (i, (plain, scaled)) in plain.iter().zip(scaled).enumerate() {
            let is_width = ["coreWidth", "glowWidth", "haloWidth"]
                .iter()
                .any(|name| STROKE_LOOK.slot(name) == Some(i));
            assert_close(scaled, if is_width { 3.0 * plain } else { *plain });
        }
    }

    /// The circle or point quad: shape, pos, size, params, fade.
    fn quad(s: &HmGuiShape, pos: Vec2, size: Vec2) -> (Shape, Vec2, Vec2, [f32; 4], [f32; 4]) {
        match s.geometry(pos, size, &HmGuiLooks::new()) {
            Some((
                shape,
                UIShapeGeometry::Quad {
                    pos,
                    size,
                    params,
                    fade,
                },
            )) => (shape, pos, size, params, fade),
            other => panic!("expected a quad, got {other:?}"),
        }
    }

    /// The stroke a shape draws and the look values it draws with.
    fn stroke(s: &HmGuiShape, pos: Vec2, size: Vec2) -> (Stroke, crate::render::StrokeLook) {
        match s.geometry(pos, size, &HmGuiLooks::new()) {
            Some((Shape::Stroke, UIShapeGeometry::Stroke { stroke, look })) => (stroke, look),
            other => panic!("expected a stroke, got {other:?}"),
        }
    }

    /// The centre and radius of the circle an arc stroke lies on.
    fn circle_of(s: &Stroke) -> (Vec2, f32) {
        let radius = 1.0 / s.curvature.abs();
        (s.start + s.normal() / s.curvature, radius)
    }

    /// The same stroke, within float rounding.
    fn assert_same(a: &Stroke, b: &Stroke) {
        assert_vec(a.start, b.start);
        assert_vec(a.direction, b.direction);
        assert_close(a.length, b.length);
        assert_close(a.curvature, b.curvature);
        assert_close(a.thickness, b.thickness);
        assert_eq!(
            (a.round_caps, a.corner, a.dash, a.fade),
            (b.round_caps, b.corner, b.dash, b.fade)
        );
    }

    fn looks(id: HmGuiLookId) -> crate::render::StrokeLook {
        *HmGuiLooks::new().values(id)
    }

    #[test]
    fn fade_windows_turn_clockwise_from_up() {
        let window = |centre, visible, gone| {
            fade_window(&HmGuiShapeFade {
                centre,
                visible,
                gone,
            })
        };

        let [centre, visible, gone, on] = window(270.0, 15.0, 32.0);
        assert_close(centre, std::f32::consts::PI); // left
        assert_close(visible, 15f32.to_radians());
        assert_close(gone, 32f32.to_radians());
        assert_eq!(on, 1.0);

        assert_close(window(0.0, 10.0, 20.0)[0], std::f32::consts::FRAC_PI_2); // up
        assert_close(window(90.0, 10.0, 20.0)[0], 0.0); // right
        assert_close(window(180.0, 10.0, 20.0)[0], 1.5 * std::f32::consts::PI); // down

        // `gone` stays past `visible`.
        let [_, visible, gone, _] = window(0.0, 20.0, 10.0);
        assert!(gone > visible);
    }

    #[test]
    fn circles_and_points_are_centred_squares() {
        // A 200 x 100 widget: radius 50, centre (110, 70).
        let pos = Vec2::new(10.0, 20.0);
        let size = Vec2::new(200.0, 100.0);

        let (shape, qpos, qsize, ..) = quad(&make(HmGuiShapeKind::Point, false), pos, size);
        assert_eq!(shape, Shape::Point);
        assert_eq!(qpos, Vec2::new(60.0, 20.0));
        assert_eq!(qsize, Vec2::splat(100.0));
        assert_eq!(
            quad(&make(HmGuiShapeKind::Point, true), pos, size).0,
            Shape::PointGlow
        );

        let (shape, _, qsize, params, fade) = quad(&make(HmGuiShapeKind::Circle, false), pos, size);
        assert_eq!(shape, Shape::CircleAlpha);
        assert_eq!(qsize, Vec2::splat(2.0 * (50.0 + 12.0)));
        assert_eq!(params[0], 50.0);
        assert_eq!(fade, [0.0; 4]);
        assert_eq!(
            quad(&make(HmGuiShapeKind::Circle, true), pos, size).0,
            Shape::Circle
        );
    }

    #[test]
    fn rings_are_full_circles_centred_in_the_widget() {
        // A 200 x 100 widget: radius 50, centre (110, 70).
        let pos = Vec2::new(10.0, 20.0);
        let size = Vec2::new(200.0, 100.0);

        let (ring, look) = stroke(&make(HmGuiShapeKind::Ring, true), pos, size);
        let (centre, radius) = circle_of(&ring);
        assert_vec(centre, Vec2::new(110.0, 70.0));
        assert_close(radius, 50.0);
        assert_close(ring.bend(), std::f32::consts::TAU);
        assert_eq!(ring.thickness, 0.0);
        assert_eq!(look, looks(HmGuiLookId::RING_GLOW));
        assert_eq!(
            stroke(&make(HmGuiShapeKind::Ring, false), pos, size).1,
            looks(HmGuiLookId::RING)
        );
    }

    #[test]
    fn bands_reach_inwards_from_the_widget_edge() {
        let pos = Vec2::ZERO;
        let size = Vec2::splat(260.0);

        // A band from radius 120 to 130: its middle at 125, its outline an
        // edge's width inside.
        let annulus = HmGuiShapeKind::Annulus { thickness: 10.0 };
        let (band, look) = stroke(&make(annulus, false), pos, size);
        let (centre, radius) = circle_of(&band);
        assert_vec(centre, Vec2::splat(130.0));
        assert_close(radius, 125.0);
        assert_close(band.thickness, 10.0 - 2.0 * ANNULUS_EDGE);
        assert_close(band.bend(), std::f32::consts::TAU);
        assert_eq!(look, looks(HmGuiLookId::ANNULUS));
        assert_eq!(
            stroke(&make(annulus, true), pos, size).1,
            looks(HmGuiLookId::ANNULUS_GLOW)
        );

        // Thicker than the radius: a full disc, its middle at half the radius.
        let disc = HmGuiShapeKind::Annulus { thickness: 500.0 };
        let (band, _) = stroke(&make(disc, false), pos, size);
        assert_close(circle_of(&band).1, 65.0);
        assert_close(band.thickness, 130.0 - 2.0 * ANNULUS_EDGE);

        // A wedge from the left an eighth of a turn up, 4 px deep: its middle
        // at radius 128, starting at the left and leaving upwards.
        let wedge = HmGuiShapeKind::Wedge {
            thickness: 4.0,
            start: 270.0,
            sweep: 45.0,
        };
        let (piece, look) = stroke(&make(wedge, false), pos, size);
        let (centre, radius) = circle_of(&piece);
        assert_vec(centre, Vec2::splat(130.0));
        assert_close(radius, 128.0);
        assert_close(piece.thickness, 4.0);
        assert_vec(piece.start, Vec2::new(2.0, 130.0));
        assert_vec(piece.direction, Vec2::new(0.0, -1.0));
        assert_close(piece.bend(), 45f32.to_radians());
        assert!(!piece.round_caps);
        assert_eq!(piece.corner, WEDGE_CORNER);
        assert_eq!(look, looks(HmGuiLookId::WEDGE));
        assert_eq!(
            stroke(&make(wedge, true), pos, size).1,
            looks(HmGuiLookId::WEDGE_GLOW)
        );
    }

    #[test]
    fn fade_windows_become_fades_along_the_arc() {
        // A wedge on radius 100 (widget 204 wide, 4 deep) from the bottom
        // clockwise round to the top, faded around the left (270).
        let mut wedge = make(
            HmGuiShapeKind::Wedge {
                thickness: 4.0,
                start: 180.0,
                sweep: 180.0,
            },
            true,
        );
        wedge.fade = Some(HmGuiShapeFade {
            centre: 270.0,
            visible: 15.0,
            gone: 40.0,
        });
        let (piece, _) = stroke(&wedge, Vec2::ZERO, Vec2::splat(204.0));
        let px = |degrees: f32| 100.0 * degrees.to_radians();
        // The left is a quarter turn from the start.
        assert_close(piece.fade.centre, px(90.0));
        assert_close(piece.fade.visible, px(15.0));
        assert_close(piece.fade.gone, px(40.0));
        assert_close(piece.fade.period, px(360.0));

        // Counter-clockwise from the top round to the bottom: the left is
        // again a quarter turn along.
        wedge.kind = HmGuiShapeKind::Wedge {
            thickness: 4.0,
            start: 0.0,
            sweep: -180.0,
        };
        let (piece, _) = stroke(&wedge, Vec2::ZERO, Vec2::splat(204.0));
        assert_close(piece.fade.centre, px(90.0));

        // Circles and points keep the window as it is.
        let mut circle = make(HmGuiShapeKind::Circle, false);
        circle.fade = wedge.fade;
        let fade = quad(&circle, Vec2::ZERO, Vec2::splat(100.0)).4;
        assert_eq!(fade, fade_window(&wedge.fade.unwrap()));

        assert!(HmGuiShapeKind::Ring.is_round());
        assert!(
            !HmGuiShapeKind::Line {
                from: Vec2::ZERO,
                to: Vec2::ONE
            }
            .is_round()
        );
    }

    #[test]
    fn triangles_and_lines_sit_at_fractions_of_the_widget() {
        let pos = Vec2::new(10.0, 20.0);
        let size = Vec2::new(40.0, 30.0);

        // The speed tag's point: tip at the left middle.
        let points = [
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 0.5),
            Vec2::new(1.0, 1.0),
        ];
        let tri = make(HmGuiShapeKind::Triangle { points }, false);
        assert_eq!(
            tri.geometry(pos, size, &HmGuiLooks::new()),
            Some((
                Shape::TriangleAlpha,
                UIShapeGeometry::Triangle {
                    points: [
                        Vec2::new(50.0, 20.0),
                        Vec2::new(10.0, 35.0),
                        Vec2::new(50.0, 50.0)
                    ],
                    pad: PAD_TRIANGLE,
                }
            ))
        );
        let tri = make(HmGuiShapeKind::Triangle { points }, true);
        let geometry = tri.geometry(pos, size, &HmGuiLooks::new());
        assert_eq!(geometry.unwrap().0, Shape::Triangle);

        // A line across the middle: a straight hairline with round ends.
        let line = HmGuiShapeKind::Line {
            from: Vec2::new(0.0, 0.5),
            to: Vec2::new(1.0, 0.5),
        };
        let (s, look) = stroke(&make(line, true), pos, size);
        assert_vec(s.start, Vec2::new(10.0, 35.0));
        assert_vec(s.point(s.length), Vec2::new(50.0, 35.0));
        assert_eq!(s.curvature, 0.0);
        assert_eq!(s.thickness, 0.0);
        assert!(s.round_caps);
        assert_eq!(look, looks(HmGuiLookId::LINE_GLOW));
        assert_eq!(
            stroke(&make(line, false), pos, size).1,
            looks(HmGuiLookId::LINE)
        );
    }

    #[test]
    fn strokes_sit_at_fractions_of_the_widget() {
        let pos = Vec2::new(10.0, 20.0);
        let size = Vec2::new(200.0, 100.0);

        let between = HmGuiShapeKind::Stroke(HmGuiStrokeForm::Between {
            from: Vec2::new(0.0, 1.0),
            to: Vec2::new(1.0, 1.0),
            bend: -90.0,
            thickness: 3.0,
        });
        let (s, look) = stroke(&make(between, false), pos, size);
        let expected = Stroke::between(Vec2::new(10.0, 120.0), Vec2::new(210.0, 120.0), -90.0, 3.0);
        assert_same(&s, &expected);
        assert_eq!(look, looks(HmGuiLookId::STROKE));

        let bent = HmGuiShapeKind::Stroke(HmGuiStrokeForm::Bent {
            start: Vec2::new(0.5, 0.5),
            direction: 90.0,
            length: 80.0,
            bend: 360.0,
            thickness: 2.0,
        });
        let (s, _) = stroke(&make(bent, false), pos, size);
        assert_same(
            &s,
            &Stroke::bent(Vec2::new(110.0, 70.0), 90.0, 80.0, 360.0, 2.0),
        );

        // The arc's middle line is on the widget's inscribed circle.
        let arc = HmGuiShapeKind::Stroke(HmGuiStrokeForm::Arc {
            start: 225.0,
            sweep: 90.0,
            thickness: 2.0,
        });
        let (s, _) = stroke(&make(arc, false), pos, size);
        assert_same(
            &s,
            &Stroke::arc(Vec2::new(110.0, 70.0), 50.0, 225.0, 90.0, 2.0),
        );
        assert!(arc.is_round() && arc.is_stroke());
        assert!(!bent.is_round() && bent.is_stroke());
        assert!(!HmGuiShapeKind::Circle.is_stroke());
    }

    #[test]
    fn strokes_carry_their_style() {
        let pos = Vec2::ZERO;
        let size = Vec2::splat(100.0);
        let arc = HmGuiShapeKind::Stroke(HmGuiStrokeForm::Arc {
            start: 0.0,
            sweep: 180.0,
            thickness: 4.0,
        });
        let mut shape = make(arc, false);
        shape.stroke.dash = StrokeDash::new(6.0, 3.0, 1.0);
        shape.stroke.round_caps = Some(true);
        shape.stroke.corner = Some(1.5);
        shape.fade = Some(HmGuiShapeFade {
            centre: 90.0,
            visible: 10.0,
            gone: 20.0,
        });
        let (s, _) = stroke(&shape, pos, size);
        assert_eq!(s.dash, StrokeDash::new(6.0, 3.0, 1.0));
        assert!(s.round_caps);
        assert_eq!(s.corner, 1.5);
        assert_close(s.fade.centre, 50.0 * 90f32.to_radians());

        // A fade along the length takes the place of the window.
        shape.stroke.fade = Some(StrokeFade::along(30.0, 5.0, 10.0));
        assert_eq!(
            stroke(&shape, pos, size).0.fade,
            StrokeFade::along(30.0, 5.0, 10.0)
        );

        // A named look, with one value of the widget's own on top.
        let mut looks = HmGuiLooks::new();
        looks.set_param("hudNeon", "glow", 0.4).unwrap();
        shape.stroke.look = looks.find("hudNeon");
        let glow = crate::render::STROKE_LOOK.slot("glow").unwrap();
        shape.stroke.overrides[glow] = Some(0.9);
        match shape.geometry(pos, size, &looks) {
            Some((_, UIShapeGeometry::Stroke { look, .. })) => assert_eq!(look[glow], 0.9),
            other => panic!("expected a stroke, got {other:?}"),
        }
    }

    #[test]
    fn nothing_to_draw() {
        let size = Vec2::splat(100.0);
        let empty = [
            (HmGuiShapeKind::Annulus { thickness: 0.0 }, size),
            (
                HmGuiShapeKind::Wedge {
                    thickness: 0.0,
                    start: 0.0,
                    sweep: 90.0,
                },
                size,
            ),
            (
                HmGuiShapeKind::Wedge {
                    thickness: 5.0,
                    start: 0.0,
                    sweep: 0.0,
                },
                size,
            ),
            (HmGuiShapeKind::Circle, Vec2::new(100.0, 0.0)),
            (HmGuiShapeKind::Ring, Vec2::new(0.0, 100.0)),
            (
                HmGuiShapeKind::Line {
                    from: Vec2::splat(0.5),
                    to: Vec2::splat(0.5),
                },
                size,
            ),
            (
                // All three corners on one line.
                HmGuiShapeKind::Triangle {
                    points: [Vec2::ZERO, Vec2::splat(0.5), Vec2::ONE],
                },
                size,
            ),
            (
                HmGuiShapeKind::Stroke(HmGuiStrokeForm::Bent {
                    start: Vec2::ZERO,
                    direction: 0.0,
                    length: 0.0,
                    bend: 90.0,
                    thickness: 2.0,
                }),
                size,
            ),
            (
                HmGuiShapeKind::Stroke(HmGuiStrokeForm::Arc {
                    start: 0.0,
                    sweep: 90.0,
                    thickness: 2.0,
                }),
                Vec2::ZERO,
            ),
        ];

        for (kind, size) in empty {
            assert_eq!(
                make(kind, false).geometry(Vec2::ZERO, size, &HmGuiLooks::new()),
                None,
                "{kind:?}"
            );
        }
    }
}
