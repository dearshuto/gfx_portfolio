#version 450

layout(location = 0) out vec4 o_Color;
layout(location = 0) in vec2 v_Uv;

layout(binding = 0) uniform texture2D u_Texture;
layout(binding = 1) uniform sampler u_Sampler;

void main()
{
    vec4 color = texture(sampler2D(u_Texture, u_Sampler), v_Uv);
    o_Color = vec4(color.rgb, 1.0);
}
