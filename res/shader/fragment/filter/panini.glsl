#version 330

in vec2 uv;
out vec4 outColor;

uniform sampler2D src;
uniform float distance;  // 0 = off, 1 = full cylindrical (0.3-0.6 typical)
uniform float scale;     // Vertical scale (1.0 = no change)

// Panini projection: maps rectilinear to cylindrical projection
// Reduces peripheral stretching at high FOV while keeping center undistorted
void main() {
    // Early out if panini is disabled
    if (distance <= 0.001) {
        outColor = texture(src, uv);
        return;
    }

    float d = distance;
    float dPlusOne = d + 1.0;
    float d2 = d * d;

    // Calculate edge compression factor to scale output to fill screen
    // At h=1 (edge), calculate how much compression occurs
    float edgeSqrtTerm = sqrt(1.0 + (d2 - 1.0) / (dPlusOne * dPlusOne));
    float edgeScale = edgeSqrtTerm;  // Scale up to compensate for edge compression

    // Convert UV [0,1] to NDC [-1,1], then scale to compensate
    vec2 ndc = (uv * 2.0 - 1.0) * edgeScale;

    float h = ndc.x;  // Horizontal position
    float v = ndc.y;  // Vertical position
    float h2 = h * h;

    // Panini projection formula
    float sqrtTerm = sqrt(1.0 + h2 * (d2 - 1.0) / (dPlusOne * dPlusOne));
    float invSqrt = 1.0 / sqrtTerm;

    // Apply horizontal compression
    float newH = h * invSqrt;

    // Scale vertical to maintain aspect ratio
    float newV = v * scale * invSqrt;

    // Convert back to UV [0,1]
    vec2 newUV = (vec2(newH, newV) + 1.0) * 0.5;

    // Clamp to valid range (edges may sample outside due to scaling)
    newUV = clamp(newUV, 0.0, 1.0);

    outColor = texture(src, newUV);
}
