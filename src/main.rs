// Declare modules for project structure
mod context;
mod graphics; // Graphics module for rendering-related utilities
mod utils; // Utility module for helper functions

// Import necessary components
use context::shader_context::ShaderContext; // Manages shader-related state and operations
use std::sync::mpsc::channel; // For inter-thread communication
use utils::watcher::watch_shader_files; // Watches shader files for changes

fn main() {
    // Define shader file paths as variables
    let vertex_shader_path = "shaders/vertex.glsl".to_string(); // Path to vertex shader
    let fragment_shader_path = "shaders/fragment.glsl".to_string(); // Path to fragment shader

    // Initialize ShaderContext with window dimensions, title, and shader paths
    let mut state = ShaderContext::new(
        800,                  // Window width
        600,                  // Window height
        "GLFW Shader Viewer", // Window title
        vertex_shader_path,   // Vertex shader path
        fragment_shader_path, // Fragment shader path
    );

    // Set up file watcher for shader files
    let (tx, rx) = channel(); // Create a channel for communication
    let _watcher = watch_shader_files(tx).expect("Failed to initialize file watcher"); // Start watching shader files

    // Main application loop
    while state.process_events() {
        // Process window events
        // Check for shader file changes
        if let Ok(_) = rx.try_recv() {
            state.reload_shaders(); // Reload shaders if files have changed
        }

        // Update timing and animation state
        state.update();

        // Render the frame
        state.render();
    }

    // _watcher will be dropped here, which is fine
}
