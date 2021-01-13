#version 330 core

in vec2 fragment_a_texture_position;
in float fragment_a_texture_face;

uniform bool u_is_lighted;
uniform bool u_is_flat_shaded;
uniform sampler2DArray u_texture_array;
uniform int u_texture_indexes[6];

out vec4 color;

void main() {
    int face = int(fragment_a_texture_face);

    float lightness;
    if (u_is_lighted) {
        if (face == 1) lightness = 1;
        if (face == 2 || face == 3) lightness = 0.9;
        if (face == 4 || face == 5) lightness = 0.7;
        if (face == 6) lightness = 0.5;
    } else {
        lightness = 1;
    }

    if (u_is_flat_shaded) {
        color = vec4(1, 1, 1, 1) * lightness;
    } else {
        color = texture(u_texture_array, vec3(fragment_a_texture_position, u_texture_indexes[face - 1])) * lightness;
    }
}
