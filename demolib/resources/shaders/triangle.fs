#version 450

layout(location = 0) out vec4 o_Color;

layout(binding = 0) uniform Material
{
    vec4 u_Color;
};

void main()
{
    o_Color = vec4(u_Color.xyz, 1.0);
}
