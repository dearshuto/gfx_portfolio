#version 450

layout(location = 0) out vec4 o_Color;
layout(location = 0) in vec3 v_Normal;

void main()
{
    o_Color = vec4(normalize(v_Normal), 1.0);
}
