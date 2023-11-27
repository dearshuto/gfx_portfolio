#version 450

layout(location = 0) out vec3 v_Normal;

layout(location = 0) in vec3 i_Position;
layout(location = 1) in vec3 i_Normal;

layout(binding = 0) uniform  View
{
    vec4 u_Mvp[4];
};

void main()
{
    vec4 position = vec4(
        dot(u_Mvp[0], vec4(i_Position, 1.0)),
        dot(u_Mvp[1], vec4(i_Position, 1.0)),
        dot(u_Mvp[2], vec4(i_Position, 1.0)),
        dot(u_Mvp[3], vec4(i_Position, 1.0)));
    gl_Position = position;
    v_Normal = i_Normal;
}
