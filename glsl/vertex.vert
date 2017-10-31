#version 140
precision lowp float;

in vec2 position;
uniform vec2 screen_size;
uniform mat3 matrix;

mat3 screen_to_ndc = mat3
(
     2.0,  0.0, 0.0,
     0.0, -2.0, 0.0,
    -1.0,  1.0, 1.0
);

vec2 transformed_position = (screen_to_ndc * matrix * vec3(position / screen_size, 1.0)).xy;

void main()
{
    gl_Position = vec4(transformed_position, 0.0, 1.0);
}
