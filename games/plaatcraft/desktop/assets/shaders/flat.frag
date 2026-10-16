#version 330 core

in vec2 fragment_a_texture_position;

uniform bool u_is_textured;
uniform vec4 u_color;
uniform sampler2D u_texture;

out vec4 color;

void main() {
    if (u_is_textured) {
        color = texture(u_texture, fragment_a_texture_position);
    } else {
        color = u_color;
    }
}
