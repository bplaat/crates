#version 330 core

in vec3 a_position;
in vec2 a_texture_position;
in float a_texture_face;

uniform mat4 u_model_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_projection_matrix;

out vec2 fragment_a_texture_position;
out float fragment_a_texture_face;

void main() {
    gl_Position = u_projection_matrix * u_view_matrix * u_model_matrix * vec4(a_position, 1);

    fragment_a_texture_position = a_texture_position;
    fragment_a_texture_face = a_texture_face;
}
