#version 330 core

in vec2 a_position;
in vec2 a_texture_position;

uniform mat4 u_model_matrix;
uniform mat4 u_projection_matrix;

out vec2 fragment_a_texture_position;

void main() {
    gl_Position = u_projection_matrix * u_model_matrix * vec4(a_position, 0, 1);

    fragment_a_texture_position = a_texture_position;
}
