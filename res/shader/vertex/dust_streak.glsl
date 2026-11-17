// Space dust streaks (SpaceDustEntity, paired with effect/dust_streak.glsl).
// Every speck is a quad whose four corners share its place in the unit cube
// (vertex_position); uv.x picks the end (0 = where it is now, 1 = where it
// was `streakTime` ago) and uv.y the side (-1, 1). The cube is wrapped around
// the camera, the two ends are projected, and the quad is built between them
// on screen, `widthPx` wide (scaled from 1080 lines to the target's height)
// and that much longer at each end, so a speck that doesn't move is a dot.
#include vertex_scene
#include math

#define camFrac (drawUser[0].xyz)  // the camera's place in the cube (0..1)
#define cell (drawUser[0].w)       // cube size (game units)
#define streak (drawUser[1].xyz)   // camera velocity * streakTime
#define opacity (drawUser[1].w)
#define widthPx1080 (drawUser[2].x) // at 1080 lines; scaled to the target's height

out float fade;

void main() {
  VS_BEGIN

  // Camera-relative place in the cube around the camera: -cell/2 .. cell/2
  vec3 rel = (fract(vertex_position - camFrac) - 0.5) * cell;

  // The camera moved by `streak` during the shutter time, so a moment ago
  // the speck was that much further along, relative to it
  vec3 head = rel;
  vec3 tail = rel + streak;

  vec4 ch = mProj * (mView * vec4(head, 1.0));
  vec4 ct = mProj * (mView * vec4(tail, 1.0));

  // Behind the camera: nothing. A tail behind it is cut where it crosses.
  const float NEAR_W = 1e-4;
  if (ch.w < NEAR_W) {
    fade = 0.0;
    gl_Position = vec4(0.0, 0.0, -2.0, 1.0);
    return;
  }
  if (ct.w < NEAR_W) {
    float t = (ch.w - NEAR_W) / (ch.w - ct.w);
    ct = mix(ch, ct, t);
  }

  // The two ends in pixels
  vec2 halfVp = 0.5 * ubo_viewport.zw;
  float widthPx = widthPx1080 * ubo_viewport.w / 1080.0;
  vec2 ph = ch.xy / ch.w * halfVp;
  vec2 pt = ct.xy / ct.w * halfVp;
  vec2 d = ph - pt;
  float lengthPx = length(d);
  vec2 along = lengthPx > 1e-3 ? d / lengthPx : vec2(1.0, 0.0);
  vec2 side = vec2(-along.y, along.x);

  // This corner: its end, pushed out by the width along and across
  float isTail = vertex_uv.x;
  vec2 p = mix(ph, pt, isTail);
  p += along * widthPx * (isTail > 0.5 ? -1.0 : 1.0);
  p += side * widthPx * vertex_uv.y;
  vec4 c = mix(ch, ct, isTail);
  gl_Position = vec4(p / halfVp * c.w, c.z, c.w);

  // Fade out towards the cube's faces (where specks wrap) and very close to
  // the camera; a longer streak spreads the same light over more pixels
  vec3 a = abs(rel) / (0.5 * cell);
  float edge = max(a.x, max(a.y, a.z));
  float faces = 1.0 - smoothstep(0.6, 1.0, edge);
  float near = smoothstep(0.0, 0.05 * cell, length(rel));
  float spread = pow(widthPx / (widthPx + lengthPx), 0.6);
  fade = opacity * faces * near * spread;

  pos = head;
  VS_END
}
