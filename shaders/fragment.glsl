#version 330 core
in vec2 TexCoord;
out vec4 FragColor;

uniform float u_time;       // Total elapsed time (seconds)
uniform float u_deltaTime;  // Time since last frame (seconds)
uniform float u_epochTime;  // System time (seconds since Unix epoch)
uniform vec2 u_resolution;  // Window size (pixels)

void main() {
    const float TAU = 6.28318530718;
    
    // Convert from TexCoord (0-1) to fragCoord style coordinates
    vec2 fragCoord = TexCoord * u_resolution;
    
    // This matches the ShaderToy mainImage function
    vec2 uv = (fragCoord - 0.5 * u_resolution) / u_resolution.y;
    float dist = length(uv);
    float baseInnerRadius = 0.25;
    float baseOuterRadius = 0.4;
    const int segments = 81;
    float segmentSize = TAU / float(segments);
    float rawAngle = atan(uv.y, uv.x);
    float angle = mod(rawAngle + TAU, TAU);
    
    // Use the more stable time source for animation
    // You could also use modulo on u_epochTime for a continuous animation
    // that stays in sync with NTP
    float animatedAngle = angle + u_time * 0.1;
    float index = floor(animatedAngle / segmentSize);
    
    // Snapped angle — continuous
    float snappedAngle = index * segmentSize;
    // Skip every other chicklet (clean logic)
    if (mod(index, 3.0) > 0.5) {
        FragColor = vec4(0.0);
        return;
    }
    // Smooth sine wave using continuous snappedAngle
    float wave = sin(snappedAngle * 7.0 + u_time * 1.0);  // 7.0 = number of wave peaks
    float radiusMod = 0.03 * wave;
    float outerRadius = baseOuterRadius + radiusMod;
    float innerRadius = baseInnerRadius + radiusMod;
    if (dist < innerRadius || dist > outerRadius) {
        FragColor = vec4(0.0);
        return;
    }
    // Color from snapped angle (rainbow)
    float hue = mod(snappedAngle, TAU) / TAU;
    float s = 1.0, v = 1.0;
    float c = v * s;
    float x = c * (1.0 - abs(mod(hue * 6.0, 2.0) - 1.0));
    float m = v - c;
    vec3 color;
    if (hue < 1.0/6.0)      color = vec3(c, x, 0.0);
    else if (hue < 2.0/6.0) color = vec3(x, c, 0.0);
    else if (hue < 3.0/6.0) color = vec3(0.0, c, x);
    else if (hue < 4.0/6.0) color = vec3(0.0, x, c);
    else if (hue < 5.0/6.0) color = vec3(x, 0.0, c);
    else                   color = vec3(c, 0.0, x);
    color += m;
    float alphaOuter = smoothstep(outerRadius, outerRadius - 0.01, dist);
    float alphaInner = smoothstep(innerRadius - 0.01, innerRadius, dist);
    float alpha = alphaOuter * alphaInner;
    FragColor = vec4(color, alpha);
}