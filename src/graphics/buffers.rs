use std::ptr;

// Creates and initializes vertex buffers for rendering a quad
pub fn create_vertex_buffers() -> (u32, u32) {
    // Vertex data: positions (3) and texture coordinates (2)
    let vertices: [f32; 30] = [
        -1.0, 1.0, 0.0, 0.0, 1.0, // Top-left
        -1.0, -1.0, 0.0, 0.0, 0.0, // Bottom-left
        1.0, -1.0, 0.0, 1.0, 0.0, // Bottom-right
        -1.0, 1.0, 0.0, 0.0, 1.0, // Top-left
        1.0, -1.0, 0.0, 1.0, 0.0, // Bottom-right
        1.0, 1.0, 0.0, 1.0, 1.0, // Top-right
    ];

    let mut vao = 0; // Vertex Array Object
    let mut vbo = 0; // Vertex Buffer Object

    unsafe {
        // Generate and bind VAO and VBO
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        // Upload vertex data to the GPU
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as isize,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        // Define vertex attribute pointers
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 20, ptr::null()); // Position attribute
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            20,
            (3 * std::mem::size_of::<f32>()) as *const _, // Texture coordinate attribute
        );
        gl::EnableVertexAttribArray(1);

        // Unbind buffers
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    (vao, vbo) // Return the VAO and VBO handles
}
