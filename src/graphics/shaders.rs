use std::ffi::CString;
use std::fs;
use std::path::Path;
use std::ptr;

// Compiles a shader from source code
pub fn compile_shader(source: &str, shader_type: u32) -> Result<u32, String> {
    unsafe {
        let shader = gl::CreateShader(shader_type); // Create a new shader object

        // Convert shader source to a C-compatible string
        let c_source = CString::new(source.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_source.as_ptr(), ptr::null());
        gl::CompileShader(shader); // Compile the shader

        // Check for compilation errors
        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

            let mut buffer = vec![0u8; len as usize];
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut i8);

            let error = String::from_utf8_lossy(&buffer);

            gl::DeleteShader(shader); // Clean up the shader object
            return Err(error.to_string());
        }

        Ok(shader) // Return the compiled shader ID
    }
}

// Compiles and links a shader program from vertex and fragment shader files
pub fn compile_shader_program(vert_path: &str, frag_path: &str) -> u32 {
    // Read shader sources from files
    let vert_source = match read_shader_source(vert_path) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Failed to read vertex shader: {}", e);
            return 0;
        }
    };

    let frag_source = match read_shader_source(frag_path) {
        Ok(source) => source,
        Err(e) => {
            eprintln!("Failed to read fragment shader: {}", e);
            return 0;
        }
    };

    // Compile vertex and fragment shaders
    let vert_shader = match compile_shader(&vert_source, gl::VERTEX_SHADER) {
        Ok(shader) => shader,
        Err(e) => {
            eprintln!("Vertex shader compilation failed: {}", e);
            return 0;
        }
    };

    let frag_shader = match compile_shader(&frag_source, gl::FRAGMENT_SHADER) {
        Ok(shader) => shader,
        Err(e) => {
            eprintln!("Fragment shader compilation failed: {}", e);
            unsafe {
                gl::DeleteShader(vert_shader);
            }
            return 0;
        }
    };

    // Link shaders into a program
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vert_shader);
        gl::AttachShader(program, frag_shader);
        gl::LinkProgram(program);

        // Check for linking errors
        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);

        if success == 0 {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);

            let mut buffer = vec![0u8; len as usize];
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buffer.as_mut_ptr() as *mut i8,
            );

            let error = String::from_utf8_lossy(&buffer);
            eprintln!("Program linking failed: {}", error);

            gl::DeleteProgram(program);
            gl::DeleteShader(vert_shader);
            gl::DeleteShader(frag_shader);

            return 0;
        }

        // Clean up shaders after linking
        gl::DetachShader(program, vert_shader);
        gl::DetachShader(program, frag_shader);
        gl::DeleteShader(vert_shader);
        gl::DeleteShader(frag_shader);

        program // Return the linked program ID
    }
}

// Creates default shaders if they do not exist
pub fn create_default_shaders(vert_path: &str, frag_path: &str) {
    // Create shaders directory if it doesn't exist
    fs::create_dir_all("shaders").unwrap();

    // Default vertex shader
    if !Path::new(vert_path).exists() {
        let default_vertex = r#"#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

void main() {
  gl_Position = vec4(aPos, 1.0);
  TexCoord = aTexCoord;
}
"#;
        fs::write(vert_path, default_vertex).expect("Failed to write default vertex shader");
    }

    // Default fragment shader
    if !Path::new(frag_path).exists() {
        let default_fragment = r#"#version 330 core
in vec2 TexCoord;
out vec4 FragColor;

uniform float u_time;       // Total elapsed time (seconds)
uniform float u_deltaTime;  // Time since last frame (seconds)
uniform float u_epochTime;  // System time (seconds since Unix epoch)
uniform vec2 u_resolution;  // Window size (pixels)

void main() {
  // Normalized coordinates (0 to 1)
  vec2 uv = TexCoord;
  
  // Time varying color
  vec3 col = 0.5 + 0.5 * cos(u_time + uv.xyx + vec3(0.0, 2.0, 4.0));
  
  // Output to screen
  FragColor = vec4(col, 1.0);
}
"#;
        fs::write(frag_path, default_fragment).expect("Failed to write default fragment shader");
    }
}

// Reads shader source code from a file
fn read_shader_source(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}
