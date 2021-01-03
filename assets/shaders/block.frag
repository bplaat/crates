#version 330 core

in vec2 fragment_a_texture_position;
in float fragment_a_texture_face;

uniform sampler2DArray u_texture_array;
uniform int u_texture_indexes[6];

out vec4 color;

void main() {
    int face = int(fragment_a_texture_face);

    float lightness;
    if (face == 0) lightness = 1;
    if (face == 1 || face == 2) lightness = 0.9;
    if (face == 3 || face == 4) lightness = 0.7;
    if (face == 5) lightness = 0.5;

    color = texture(u_texture_array, vec3(fragment_a_texture_position, u_texture_indexes[face])) * lightness;
}
