#version 450

#define LOOP_COUNT (360)

layout(location = 0) out vec4 o_Color;
layout(location = 0) in vec2 v_NormalizedFragCoord;

void main() {
  int count = 0;
  vec2 offset = vec2(-0.5, 0.0) + v_NormalizedFragCoord;
  vec2 z = vec2(0.0);

  for (int i = 0; i < LOOP_COUNT; ++i) {
    ++count;
    if (length(z) > 2.0) {
      break;
    }

    z = vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + offset;
  }

  float h = log(float(count) / LOOP_COUNT);
  float s = 0.9;
  float v = 0.7;
  vec3 color =
      ((clamp(abs(fract(h + vec3(0, 2, 1) / 3.) * 6. - 3.) - 1., 0., 1.) - 1.) *
           s +
       1.) *
      v;
  o_Color = vec4(color, 1.0);
}
