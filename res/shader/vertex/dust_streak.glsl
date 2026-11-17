// Space dust streaks (SpaceDustEntity, paired with effect/dust_streak.glsl).
// Every speck is a strip whose corners all share its place in the unit cube
// (vertex_position); uv.x is how far back along the streak the corner is
// (0 = where the speck is now, 1 = the far end) and uv.y the side (-1, 1).
// The cube is wrapped around the camera; each corner's point on the streak is
// projected, pushed out of the bubble (a disc on screen around the ship), and
// moved sideways by the width, across the direction the streak runs there.
// The strip is `widthPx` wide (scaled from 1080 lines to the target's height)
// and that much longer at each end, so a speck that doesn't move is a dot.
#include vertex_scene
#include math

#define camFrac (drawUser[0].xyz)    // the camera's place in the cube (0..1)
#define cell (drawUser[0].w)         // cube size (game units)
#define streak (drawUser[1].xyz)     // where the speck was, relative to now
#define opacity (drawUser[1].w)
#define widthPx1080 (drawUser[2].x)
#define brightness (drawUser[2].y)
#define segments (drawUser[2].z)
#define bubbleNdc (drawUser[4].xy)   // the ship on screen
#define bubbleRadius (drawUser[4].z) // fraction of the screen height
#define bubbleStrength (drawUser[4].w)
#define bubbleCurl (drawUser[5].x)   // radians the bubble turns points at its centre
#define dustShift (drawUser[5].yz)   // NDC the whole dust moves on screen (third person)

out float fade;

const float NEAR_W = 1e-4;

// A point on screen (pixels from the centre), moved by the dust's shift (so
// the streaks converge on the ship in third person) and pushed out of the
// bubble: at distance d from the ship it moves to sqrt(d^2 + B^2), so
// nothing stays inside radius B and points far away hardly move. It is also
// turned around the ship, by the curl times B^2 / (d^2 + B^2) (all of it at
// the centre, half at the rim, hardly any far away), so streaks bend around
// the bubble.
vec2 onScreen(vec4 c, vec2 halfVp) {
  vec2 p = (c.xy / c.w + dustShift) * halfVp;
  float B = bubbleRadius * ubo_viewport.w * bubbleStrength;
  if (B <= 0.0) return p;
  vec2 ship = bubbleNdc * halfVp;
  vec2 q = p - ship;
  float d = length(q);
  vec2 dir = d > 1e-3 ? q / d : vec2(0.0, 1.0);
  float d2 = d * d;
  float B2 = B * B;
  float turn = bubbleCurl * B2 / (d2 + B2);
  float cs = cos(turn);
  float sn = sin(turn);
  dir = vec2(cs * dir.x - sn * dir.y, sn * dir.x + cs * dir.y);
  return ship + dir * sqrt(d2 + B2);
}

void main() {
  VS_BEGIN
  vec2 halfVp = 0.5 * ubo_viewport.zw;
  float widthPx = widthPx1080 * ubo_viewport.w / 1080.0;

  // Camera-relative place in the cube around the camera: -cell/2 .. cell/2
  vec3 rel = (fract(vertex_position - camFrac) - 0.5) * cell;

  // The two ends in clip space; clip coordinates are linear in the position,
  // so every point between is a mix of them
  vec4 c0 = mProj * (mView * vec4(rel, 1.0));
  vec4 c1 = mProj * (mView * vec4(rel + streak, 1.0));

  // Behind the camera: nothing. A streak running behind it is cut short.
  if (c0.w < NEAR_W) {
    fade = 0.0;
    gl_Position = vec4(0.0, 0.0, -2.0, 1.0);
    return;
  }
  float tMax = c1.w < NEAR_W ? (c0.w - NEAR_W) / (c0.w - c1.w) : 1.0;

  // This corner's point, and a neighbour on the streak for its direction
  float t = vertex_uv.x * tMax;
  float dt = tMax / segments;
  float tn = vertex_uv.x < 0.999 ? t + dt : t - dt;
  vec4 ct = mix(c0, c1, t);
  vec2 p = onScreen(ct, halfVp);
  vec2 pn = onScreen(mix(c0, c1, tn), halfVp);
  vec2 toHead = tn > t ? p - pn : pn - p; // pointing to where the speck is now
  float segLength = length(toHead);
  vec2 along = segLength > 1e-3 ? toHead / segLength : vec2(1.0, 0.0);
  vec2 side = vec2(-along.y, along.x);

  p += side * widthPx * vertex_uv.y;
  if (vertex_uv.x < 0.001) p += along * widthPx;
  if (vertex_uv.x > 0.999) p -= along * widthPx;
  gl_Position = vec4(p / halfVp * ct.w, ct.z, ct.w);

  // Fade out towards the cube's faces (where specks wrap) and very close to
  // the camera; a longer streak spreads the same light over more pixels
  vec3 a = abs(rel) / (0.5 * cell);
  float edge = max(a.x, max(a.y, a.z));
  float faces = 1.0 - smoothstep(0.6, 1.0, edge);
  float near = smoothstep(0.0, 0.05 * cell, length(rel));
  float lengthPx = distance(onScreen(c0, halfVp), onScreen(mix(c0, c1, tMax), halfVp));
  float spread = pow(widthPx / (widthPx + lengthPx), 0.5);
  fade = opacity * brightness * faces * near * spread;

  pos = rel;
  VS_END
}
