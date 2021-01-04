#version 330 core

in vec2 fragment_a_texture_position;

uniform sampler2D u_texture;

out vec4 color;

void main() {
    color = texture(u_texture, fragment_a_texture_position);
}
