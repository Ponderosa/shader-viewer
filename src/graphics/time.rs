use std::time::{Duration, SystemTime};

// Manages timing information for the application
pub struct TimeState {
    pub start_time: SystemTime,      // Time when the application started
    pub last_frame_time: SystemTime, // Time of the last frame
    pub delta_time: f32,             // Time elapsed since the last frame (in seconds)
    pub total_time: f32, // Total time elapsed since the application started (in seconds)
}

impl TimeState {
    // Creates a new TimeState instance with the current time
    pub fn new() -> Self {
        let now = SystemTime::now();
        Self {
            start_time: now,
            last_frame_time: now,
            delta_time: 0.0,
            total_time: 0.0,
        }
    }

    // Updates the timing information for the current frame
    pub fn update(&mut self) {
        let now = SystemTime::now();
        self.delta_time = now
            .duration_since(self.last_frame_time)
            .unwrap_or(Duration::from_secs(0))
            .as_secs_f32(); // Calculate time since last frame
        self.total_time = now
            .duration_since(self.start_time)
            .unwrap_or(Duration::from_secs(0))
            .as_secs_f32(); // Calculate total elapsed time
        self.last_frame_time = now; // Update the last frame time
    }
}
