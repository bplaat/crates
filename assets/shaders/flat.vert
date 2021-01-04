#version 330 core

in vec2 a_position;
in vec2 a_texture_position;

uniform mat4 u_matrix;

out vec2 fragment_a_texture_position;

void main() {
    gl_Position = u_matrix * vec4(a_position, 0, 1);

    fragment_a_texture_position = a_texture_position;
}
