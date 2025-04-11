#version 330 core

// Input attributes from the vertex buffer
layout (location = 0) in vec3 aPos; // Vertex position
layout (location = 1) in vec2 aTexCoord; // Texture coordinate

// Output to the fragment shader
out vec2 TexCoord; // Pass texture coordinate to fragment shader

void main() {
    // Set the position of the vertex in clip space
    gl_Position = vec4(aPos, 1.0);
    // Pass the texture coordinate to the fragment shader
    TexCoord = aTexCoord;
}
