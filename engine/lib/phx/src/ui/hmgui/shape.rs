use glam::Vec2;

use crate::render::{Color, Shape, UIRenderer, UIShapeGeometry};

/// What a shape widget draws.
///
/// The round kinds (annulus, circle, point, ring, wedge) are centred in their
/// widget and as wide as its smaller side: the widget's size is the shape's
/// size. Triangles and lines place their points at fractions of the widget
/// (0, 0 = top left, 1, 1 = bottom right), like CSS `clip-path: polygon()`.
/// Glow reaches past the widget's edge (like a CSS `box-shadow`).
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
}

impl HmGuiShapeKind {
    /// Centred in the widget, so it can take a fade window.
    pub fn is_round(&self) -> bool {
        !matches!(self, Self::Triangle { .. } | Self::Line { .. })
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

/// A shape widget: one of the engine's `ui/*` shapes in the layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HmGuiShape {
    pub kind: HmGuiShapeKind,
    pub color: Color,
    /// Draw additive (brightens what is behind it; neon) instead of alpha
    /// blended (can also darken it, e.g. a dark halo under a glowing shape).
    pub glow: bool,
    /// Round shapes only.
    pub fade: Option<HmGuiShapeFade>,
}

/// Room for the soft glow tails of `ring.glsl` and of the glowing annulus,
/// down to well under 1/255 at the quad's edge.
const PAD_RING: f32 = 64.0;
/// Room for the glow of `wedge.glsl` (as `DrawEx.Wedge`).
const PAD_WEDGE: f32 = 32.0;
/// Room for the plain annulus' 2 px feather.
const PAD_ANNULUS: f32 = 4.0;
/// Room for the glow of `triangle.glsl` (as `DrawEx.Tri`).
const PAD_TRIANGLE: f32 = 32.0;
/// Room for the glow of `line.glsl` (as `DrawEx.Line`).
const PAD_LINE: f32 = 64.0;

impl HmGuiShape {
    /// The engine shape and where to draw it for a widget at `pos` with
    /// `size`, or `None` when there is nothing to draw (an empty widget, no
    /// thickness, no sweep, a line or triangle without length or area).
    pub fn geometry(&self, pos: Vec2, size: Vec2) -> Option<(Shape, UIShapeGeometry)> {
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
            HmGuiShapeKind::Line { from, to } => {
                let (from, to) = (pos + from * size, pos + to * size);
                if from.distance_squared(to) < 1e-6 {
                    return None;
                }
                let shape = if self.glow {
                    Shape::LineGlow
                } else {
                    Shape::LineAlpha
                };
                let pad = PAD_LINE;
                Some((shape, UIShapeGeometry::Line { from, to, pad }))
            }
            _ => self.round_geometry(pos, size),
        }
    }

    fn round_geometry(&self, pos: Vec2, size: Vec2) -> Option<(Shape, UIShapeGeometry)> {
        let radius = 0.5 * size.x.min(size.y);
        if radius <= 0.0 {
            return None;
        }

        let (shape, pad, params) = match self.kind {
            HmGuiShapeKind::Annulus { thickness } => {
                if thickness <= 0.0 {
                    return None;
                }
                let inner = (radius - thickness).max(0.0);
                if self.glow {
                    (Shape::AnnulusGlow, PAD_RING, [inner, radius, 0.0, 0.0])
                } else {
                    (Shape::Annulus, PAD_ANNULUS, [inner, radius, 0.0, 0.0])
                }
            }
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
            HmGuiShapeKind::Ring => {
                let shape = if self.glow {
                    Shape::RingGlow
                } else {
                    Shape::Ring
                };
                (shape, PAD_RING, [radius, 0.0, 0.0, 0.0])
            }
            HmGuiShapeKind::Wedge {
                thickness,
                start,
                sweep,
            } => {
                if thickness <= 0.0 || sweep == 0.0 {
                    return None;
                }
                let inner = (radius - thickness).max(0.0);
                let (to, tw) = wedge_turns(start, sweep);
                let shape = if self.glow {
                    Shape::Wedge
                } else {
                    Shape::WedgeAlpha
                };
                (shape, PAD_WEDGE, [inner, radius, to, tw])
            }
            HmGuiShapeKind::Triangle { .. } | HmGuiShapeKind::Line { .. } => {
                unreachable!("not a round shape")
            }
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

    pub fn draw(&self, renderer: &mut UIRenderer, pos: Vec2, size: Vec2) {
        if let Some((shape, geometry)) = self.geometry(pos, size) {
            renderer.shape(shape, geometry, self.color);
        }
    }

    // For testing.
    #[allow(dead_code)]
    pub(crate) fn dump(&self, ident: usize) {
        let ident_str = crate::ui::hmgui::IDENT.repeat(ident).to_string();

        println!("{ident_str}- kind:  {:?}", self.kind);
        println!("{ident_str}- color: {:?}", self.color);
        println!("{ident_str}- glow:  {}", self.glow);
        println!("{ident_str}- fade:  {:?}", self.fade);
    }
}

/// Wedge angles in degrees (0 = up, clockwise, from `start` to
/// `start + sweep`) as the engine's `Wedge` takes them: the centre angle and
/// the width, in turns (0 = right, counter-clockwise).
pub fn wedge_turns(start: f32, sweep: f32) -> (f32, f32) {
    let centre = start + 0.5 * sweep;
    let to = (0.25 - centre / 360.0).rem_euclid(1.0);
    let tw = (sweep.abs() / 360.0).min(1.0);
    (to, tw)
}

/// A fade window in degrees (0 = up, clockwise) as the round shaders take it
/// (`Renderer::imm_shape_faded`): `[centre, visible, gone, 1]` in radians,
/// 0 = right, counter-clockwise. `gone` is kept past `visible`, so the ramp
/// between them is never empty.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-5,
            "expected {expected}, got {actual}"
        );
    }

    fn make(kind: HmGuiShapeKind, glow: bool) -> HmGuiShape {
        HmGuiShape {
            kind,
            color: Color::WHITE,
            glow,
            fade: None,
        }
    }

    /// The round shape's quad: shape, pos, size, params, fade.
    fn quad(s: &HmGuiShape, pos: Vec2, size: Vec2) -> (Shape, Vec2, Vec2, [f32; 4], [f32; 4]) {
        match s.geometry(pos, size) {
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

    #[test]
    fn wedge_angles_turn_clockwise_from_up() {
        // (start, sweep) in degrees -> (centre, width) in turns
        let cases = [
            ((-10.0, 20.0), (0.25, 20.0 / 360.0)), // around up
            ((80.0, 20.0), (0.0, 20.0 / 360.0)),   // around right
            ((170.0, 20.0), (0.75, 20.0 / 360.0)), // around down
            ((260.0, 20.0), (0.5, 20.0 / 360.0)),  // around left
            ((270.0, 45.0), (0.4375, 0.125)),      // from left, an eighth up
            ((280.0, -20.0), (0.5, 20.0 / 360.0)), // a negative sweep
            ((0.0, 720.0), (0.25, 1.0)),           // more than a full turn
        ];

        for ((start, sweep), (to, tw)) in cases {
            let (actual_to, actual_tw) = wedge_turns(start, sweep);
            assert_close(actual_to, to);
            assert_close(actual_tw, tw);
        }
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
    fn round_shapes_are_centred_squares_with_room_for_their_glow() {
        // A 200 x 100 widget: radius 50, centre (110, 70).
        let pos = Vec2::new(10.0, 20.0);
        let size = Vec2::new(200.0, 100.0);

        let (shape, qpos, qsize, params, fade) = quad(&make(HmGuiShapeKind::Ring, true), pos, size);
        assert_eq!(shape, Shape::RingGlow);
        assert_eq!(qpos, Vec2::new(110.0 - 114.0, 70.0 - 114.0));
        assert_eq!(qsize, Vec2::splat(228.0));
        assert_eq!(params, [50.0, 0.0, 0.0, 0.0]);
        assert_eq!(fade, [0.0; 4]);

        let (shape, qpos, qsize, ..) = quad(&make(HmGuiShapeKind::Point, false), pos, size);
        assert_eq!(shape, Shape::Point);
        assert_eq!(qpos, Vec2::new(60.0, 20.0));
        assert_eq!(qsize, Vec2::splat(100.0));

        let (shape, _, qsize, params, _) = quad(&make(HmGuiShapeKind::Circle, false), pos, size);
        assert_eq!(shape, Shape::CircleAlpha);
        assert_eq!(qsize, Vec2::splat(2.0 * (50.0 + 12.0)));
        assert_eq!(params[0], 50.0);
    }

    #[test]
    fn bands_reach_inwards_from_the_widget_edge() {
        let pos = Vec2::ZERO;
        let size = Vec2::splat(260.0);

        let annulus = HmGuiShapeKind::Annulus { thickness: 10.0 };
        let (shape, qpos, _, params, _) = quad(&make(annulus, false), pos, size);
        assert_eq!(shape, Shape::Annulus);
        assert_eq!(params, [120.0, 130.0, 0.0, 0.0]);
        assert_eq!(qpos, Vec2::splat(-4.0));

        let (shape, qpos, ..) = quad(&make(annulus, true), pos, size);
        assert_eq!(shape, Shape::AnnulusGlow);
        assert_eq!(qpos, Vec2::splat(-64.0));

        // Thicker than the radius: a full disc.
        let disc = HmGuiShapeKind::Annulus { thickness: 500.0 };
        assert_eq!(quad(&make(disc, false), pos, size).3[0], 0.0);

        let wedge = HmGuiShapeKind::Wedge {
            thickness: 4.0,
            start: 270.0,
            sweep: 45.0,
        };
        let (shape, qpos, _, params, _) = quad(&make(wedge, false), pos, size);
        assert_eq!(shape, Shape::WedgeAlpha);
        assert_eq!(params[0], 126.0);
        assert_eq!(params[1], 130.0);
        assert_close(params[2], 0.4375);
        assert_close(params[3], 0.125);
        assert_eq!(qpos, Vec2::splat(-32.0));
        assert_eq!(quad(&make(wedge, true), pos, size).0, Shape::Wedge);
    }

    #[test]
    fn round_shapes_carry_their_fade_window() {
        let mut ring = make(HmGuiShapeKind::Ring, false);
        ring.fade = Some(HmGuiShapeFade {
            centre: 270.0,
            visible: 15.0,
            gone: 32.0,
        });

        let fade = quad(&ring, Vec2::ZERO, Vec2::splat(100.0)).4;
        assert_eq!(fade, fade_window(&ring.fade.unwrap()));
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
            tri.geometry(pos, size),
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
        assert_eq!(tri.geometry(pos, size).unwrap().0, Shape::Triangle);

        let line = HmGuiShapeKind::Line {
            from: Vec2::new(0.0, 0.5),
            to: Vec2::new(1.0, 0.5),
        };
        assert_eq!(
            make(line, true).geometry(pos, size),
            Some((
                Shape::LineGlow,
                UIShapeGeometry::Line {
                    from: Vec2::new(10.0, 35.0),
                    to: Vec2::new(50.0, 35.0),
                    pad: PAD_LINE,
                }
            ))
        );
        assert_eq!(
            make(line, false).geometry(pos, size).unwrap().0,
            Shape::LineAlpha
        );
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
        ];

        for (kind, size) in empty {
            assert_eq!(
                make(kind, false).geometry(Vec2::ZERO, size),
                None,
                "{kind:?}"
            );
        }
    }
}
