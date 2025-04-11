use std::ffi::CString;
use std::sync::mpsc::Receiver;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::graphics::{
    buffers::create_vertex_buffers,
    shaders::{compile_shader_program, create_default_shaders},
    time::TimeState,
};
use glfw::{Action, Context, Key};

// Manages the OpenGL context, window, and rendering state
pub struct ShaderContext {
    pub window: glfw::Window,                       // GLFW window instance
    pub events: Receiver<(f64, glfw::WindowEvent)>, // Event receiver for window events
    pub glfw: glfw::Glfw,                           // GLFW instance
    pub shader_program: u32,                        // OpenGL shader program ID
    pub vao: u32,                                   // Vertex Array Object ID
    pub vbo: u32,                                   // Vertex Buffer Object ID
    pub time_state: TimeState,                      // Timing state for animations
    pub vertex_shader_path: String,                 // Path to the vertex shader file
    pub fragment_shader_path: String,               // Path to the fragment shader file
}

impl ShaderContext {
    // Creates a new ShaderContext instance
    pub fn new(
        width: u32,
        height: u32,
        title: &str,
        vertex_shader_path: String,
        fragment_shader_path: String,
    ) -> Self {
        // Initialize GLFW
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

        // Set OpenGL version and profile
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));

        // Enable double buffering
        glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));

        // macOS compatibility
        #[cfg(target_os = "macos")]
        glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

        // Create a windowed mode window and its OpenGL context
        let (mut window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");

        // Enable VSync
        window.make_current();
        glfw.set_swap_interval(glfw::SwapInterval::Sync(1));
        window.set_key_polling(true);
        window.set_framebuffer_size_polling(true);

        // Load OpenGL function pointers
        gl::load_with(|s| window.get_proc_address(s) as *const _);

        // Create and bind VAO and VBO
        let (vao, vbo) = create_vertex_buffers();

        // Create default shaders if they don't exist
        create_default_shaders(&vertex_shader_path, &fragment_shader_path);

        // Compile and link shaders into a program
        let shader_program = compile_shader_program(&vertex_shader_path, &fragment_shader_path);

        ShaderContext {
            window,
            events,
            glfw,
            shader_program,
            vao,
            vbo,
            time_state: TimeState::new(),
            vertex_shader_path,
            fragment_shader_path,
        }
    }

    // Reloads shaders and updates the shader program
    pub fn reload_shaders(&mut self) {
        println!("Reloading shaders...");

        // Recompile shader program
        let new_program =
            compile_shader_program(&self.vertex_shader_path, &self.fragment_shader_path);

        // Replace the old program if compilation succeeds
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

    // Updates the timing state for animations
    pub fn update(&mut self) {
        self.time_state.update();
    }

    // Renders a frame
    pub fn render(&mut self) {
        unsafe {
            // Clear the screen
            gl::ClearColor(0.1, 0.1, 0.1, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Use the shader program
            gl::UseProgram(self.shader_program);

            // Update time uniforms
            let time_loc = gl::GetUniformLocation(
                self.shader_program,
                CString::new("u_time").unwrap().as_ptr(),
            );
            gl::Uniform1f(time_loc, self.time_state.total_time);

            let delta_loc = gl::GetUniformLocation(
                self.shader_program,
                CString::new("u_deltaTime").unwrap().as_ptr(),
            );
            gl::Uniform1f(delta_loc, self.time_state.delta_time);

            let epoch_loc = gl::GetUniformLocation(
                self.shader_program,
                CString::new("u_epochTime").unwrap().as_ptr(),
            );
            let epoch_secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f32();
            gl::Uniform1f(epoch_loc, epoch_secs);

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

        // Swap front and back buffers - Blocking since we have vsync enabled
        self.window.swap_buffers();
    }

    // Processes window events and checks if the window should close
    pub fn process_events(&mut self) -> bool {
        self.glfw.poll_events(); // Poll for events

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

        if self.window.should_close() {
            should_close = true;
        }

        !should_close
    }
}

impl Drop for ShaderContext {
    // Cleans up OpenGL resources when the ShaderContext is dropped
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.shader_program);
            gl::DeleteBuffers(1, &self.vbo);
            gl::DeleteVertexArrays(1, &self.vao);
        }
    }
}
