#version 450

layout(location = 0) out vec2 v_Uv;
layout(location = 0) in vec2 i_Position;

void main()
{
    gl_Position = vec4(i_Position, 0.5, 1.0);
    v_Uv = (i_Position + 1.0) / 2.0;
    v_Uv.y = 1.0 - v_Uv.y;
}
