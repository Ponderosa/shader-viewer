use std::ffi::CString;
use std::fs;
use std::path::Path;
use std::ptr;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use glfw::{Action, Context, Key};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};

const VERTEX_SHADER_PATH: &str = "shaders/vertex.glsl";
const FRAGMENT_SHADER_PATH: &str = "shaders/fragment.glsl";

// Struct to hold our application state
struct State {
    window: glfw::Window,
    events: Receiver<(f64, glfw::WindowEvent)>,
    glfw: glfw::Glfw,
    shader_program: u32,
    vao: u32,
    vbo: u32,
    start_time: Instant,
    vertex_shader_path: String,
    fragment_shader_path: String,
}

impl State {
    fn new(width: u32, height: u32, title: &str) -> Self {
        // Initialize GLFW
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        // Set OpenGL version - explicitly request 3.3
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));

        // Set this to true for macOS - important for compatibility
        #[cfg(target_os = "macos")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        // Create a windowed mode window and its OpenGL context
        let (mut window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");

        // Make the window's context current
        window.make_current();
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);

        // Load OpenGL function pointers
        gl::load_with(|s| window.get_proc_address(s) as *const _);

        // Create and bind VAO and VBO
        let (vao, vbo) = unsafe {
            let mut vao = 0;
            let mut vbo = 0;

            // Create and bind VAO
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // Create and bind VBO
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            // Define vertex data (full-screen quad)
            let vertices: [f32; 30] = [
                // Positions (3) // Texture coords (2)
                -1.0, 1.0, 0.0, 0.0, 1.0, // Top-left
                -1.0, -1.0, 0.0, 0.0, 0.0, // Bottom-left
                1.0, -1.0, 0.0, 1.0, 0.0, // Bottom-right
                -1.0, 1.0, 0.0, 0.0, 1.0, // Top-left
                1.0, -1.0, 0.0, 1.0, 0.0, // Bottom-right
                1.0, 1.0, 0.0, 1.0, 1.0, // Top-right
            ];

            // Upload vertex data
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // Set vertex attribute pointers
            // Position attribute
            gl::VertexAttribPointer(
                0,                                       // attribute index
                3,                                       // size (vec3)
                gl::FLOAT,                               // type
                gl::FALSE,                               // normalized
                (5 * std::mem::size_of::<f32>()) as i32, // stride (5 floats per vertex)
                ptr::null(),                             // offset
            );
            gl::EnableVertexAttribArray(0);

            // Texture coordinate attribute
            gl::VertexAttribPointer(
                1,                                            // attribute index
                2,                                            // size (vec2)
                gl::FLOAT,                                    // type
                gl::FALSE,                                    // normalized
                (5 * std::mem::size_of::<f32>()) as i32,      // stride (5 floats per vertex)
                (3 * std::mem::size_of::<f32>()) as *const _, // offset (after 3 floats)
            );
            gl::EnableVertexAttribArray(1);

            // Unbind
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);

            (vao, vbo)
        };

        // Create directories and default shaders if they don't exist
        create_default_shaders();

        // Load shaders and create program
        let shader_program = compile_shader_program(VERTEX_SHADER_PATH, FRAGMENT_SHADER_PATH);

        State {
            window,
            events,
            glfw,
            shader_program,
            vao,
            vbo,
            start_time: Instant::now(),
            vertex_shader_path: VERTEX_SHADER_PATH.to_string(),
            fragment_shader_path: FRAGMENT_SHADER_PATH.to_string(),
        }
    }

    fn reload_shaders(&mut self) {
        println!("Reloading shaders...");

        // Recompile shader program
        let new_program =
            compile_shader_program(&self.vertex_shader_path, &self.fragment_shader_path);

        // If successful, replace the old program
        if new_program != 0 {
            unsafe {
                gl::DeleteProgram(self.shader_program);
            }
            self.shader_program = new_program;
            println!("Shaders reloaded successfully");
        } else {
            println!("Failed to reload shaders");
        }
    }

    fn render(&mut self) {
        unsafe {
            // Clear the screen
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Use shader program
            gl::UseProgram(self.shader_program);

            // Update uniforms
            let elapsed = self.start_time.elapsed().as_secs_f32();
            let time_loc = gl::GetUniformLocation(
                self.shader_program,
                CString::new("u_time").unwrap().as_ptr(),
            );
            gl::Uniform1f(time_loc, elapsed);

            // Update resolution uniform
            let (width, height) = self.window.get_framebuffer_size();
            let resolution_loc = gl::GetUniformLocation(
                self.shader_program,
                CString::new("u_resolution").unwrap().as_ptr(),
            );
            gl::Uniform2f(resolution_loc, width as f32, height as f32);

            // Draw the quad
            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::BindVertexArray(0);
        }

        // Swap front and back buffers
        self.window.swap_buffers();
    }

    fn process_events(&mut self) -> bool {
        // Poll for and process events
        self.glfw.poll_events();

        // Process received events
        let mut should_close = false;
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.window.set_should_close(true);
                    should_close = true;
                }
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    // Update the viewport
                    unsafe {
                        gl::Viewport(0, 0, width, height);
                    }
                }
                _ => {}
            }
        }

        // Check if the window should close
        if self.window.should_close() {
            should_close = true;
        }

        !should_close
    }
}

impl Drop for State {
    fn drop(&mut self) {
        // Clean up OpenGL resources
        unsafe {
            gl::DeleteProgram(self.shader_program);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}

// Read shader source from file
fn read_shader_source(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

// Compile a shader
fn compile_shader(source: &str, shader_type: u32) -> Result<u32, String> {
    unsafe {
        let shader = gl::CreateShader(shader_type);

        // Convert shader source to C string
        let c_source = CString::new(source.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_source.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Check for compile errors
        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);

        if success == 0 {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

            let mut buffer = vec![0u8; len as usize];
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut i8);

            let error = String::from_utf8_lossy(&buffer);

            gl::DeleteShader(shader);
            return Err(error.to_string());
        }

        Ok(shader)
    }
}

// Compile and link shader program
fn compile_shader_program(vert_path: &str, frag_path: &str) -> u32 {
    // Read shader sources
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

    // Compile shaders
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

    // Create and link program
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

        // Detach and delete shaders (they're no longer needed after linking)
        gl::DetachShader(program, vert_shader);
        gl::DetachShader(program, frag_shader);
        gl::DeleteShader(vert_shader);
        gl::DeleteShader(frag_shader);

        program
    }
}

// Create default shader files if they don't exist
fn create_default_shaders() {
    // Create shaders directory if it doesn't exist
    fs::create_dir_all("shaders").unwrap();

    // Default vertex shader
    if !Path::new(VERTEX_SHADER_PATH).exists() {
        let default_vertex = r#"#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec2 aTexCoord;

out vec2 TexCoord;

void main() {
    gl_Position = vec4(aPos, 1.0);
    TexCoord = aTexCoord;
}
"#;
        fs::write(VERTEX_SHADER_PATH, default_vertex)
            .expect("Failed to write default vertex shader");
    }

    // Default fragment shader
    if !Path::new(FRAGMENT_SHADER_PATH).exists() {
        let default_fragment = r#"#version 330 core
in vec2 TexCoord;
out vec4 FragColor;

uniform float u_time;
uniform vec2 u_resolution;

void main() {
    // Normalized coordinates (0 to 1)
    vec2 uv = TexCoord;
    
    // Time varying color
    vec3 col = 0.5 + 0.5 * cos(u_time + uv.xyx + vec3(0.0, 2.0, 4.0));
    
    // Output to screen
    FragColor = vec4(col, 1.0);
}
"#;
        fs::write(FRAGMENT_SHADER_PATH, default_fragment)
            .expect("Failed to write default fragment shader");
    }
}

// File watching setup
fn watch_shader_files(sender: mpsc::Sender<()>) -> notify::Result<RecommendedWatcher> {
    // Create a watcher
    let mut watcher = RecommendedWatcher::new(
        move |result: Result<notify::Event, notify::Error>| {
            match result {
                Ok(event) => {
                    if event.kind.is_modify() {
                        // Delay a bit to avoid multiple events for a single save
                        thread::sleep(Duration::from_millis(50));
                        let _ = sender.send(());
                    }
                }
                Err(e) => println!("Watch error: {:?}", e),
            }
        },
        Config::default(),
    )?;

    // Watch the shaders directory
    watcher.watch(Path::new("shaders"), RecursiveMode::Recursive)?;

    Ok(watcher)
}

fn main() {
    // Create our application state
    let mut state = State::new(800, 600, "GLFW Shader Viewer");

    // Set up file watcher
    let (sender, receiver) = mpsc::channel();
    let _watcher = match watch_shader_files(sender) {
        Ok(watcher) => Some(watcher),
        Err(e) => {
            eprintln!("Failed to watch shader files: {}", e);
            None
        }
    };

    // Main loop
    while state.process_events() {
        // Check for shader file changes
        if receiver.try_recv().is_ok() {
            state.reload_shaders();
        }

        // Render
        state.render();
    }

    // _watcher will be dropped here, which is fine
}
