layout(location = 0) out vec4 o_Color;

/*
layout(binding = 0) uniform MaterialBuffer
{
    vec4 u_Color;
};
*/

void main()
{
    // o_Color = vec4(u_Color.rgb, 1.0);
    o_Color = vec4(1.0, 0.0, 0.0, 1.0);
}
