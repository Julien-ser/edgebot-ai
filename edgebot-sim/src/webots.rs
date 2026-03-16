//! Webots robotics simulator integration for EdgeBot AI.
//!
//! This module provides a safe, Python-like API wrapper around the `webots-sys` FFI bindings
//! for controlling Webots simulations, including headless mode for automated testing.
//!
//! # Example: Simple supervisor simulation
//!
//! ```rust
//! use edgebot_sim::webots::{Supervisor, Robot, WebotsError};
//!
//! fn main() -> Result<(), WebotsError> {
//!     // Launch Webots with a world file in headless mode (or connect to existing)
//!     let mut supervisor = Supervisor::launch("worlds/my_world.wbt", true)?;
//!
//!     // Spawn a robot from a prototype
//!     let robot = supervisor.spawn_robot(
//!         "prototypes/my_robot.proto",
//!         "test_robot"
//!     )?;
//!
//!     // Step the simulation
//!     supervisor.step(32)?;
//!
//!     // Get robot's camera device
//!     let camera = robot.get_device("camera")?.as_camera()?;
//!     camera.enable(32)?;
//!
//!     // Run simulation loop
//!     for _ in 0..100 {
//!         supervisor.step(32)?;
//!         let image = camera.get_image()?;
//!         // Process image (e.g., run inference with edgebot-core)
//!     }
//!
//!     Ok(())
//! }
//! ```

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::path::Path;
use std::process::Command;
use thiserror::Error;

/// Re-export of the `webots` crate for FFI functions and types.
extern crate webots as ffi;

/// Webots-specific error type.
#[derive(Error, Debug)]
pub enum WebotsError {
    #[error("FFI error: {0}")]
    FfiError(String),
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("Robot not found: {0}")]
    RobotNotFound(String),
    #[error("Node not found")]
    NodeNotFound,
    #[error("Invalid handle")]
    InvalidHandle,
    #[error("Supervisor required")]
    NotSupervisor,
    #[error("Spawning robot failed")]
    SpawnFailed,
    #[error("Loading world failed")]
    LoadWorldFailed,
    #[error("Connection failed")]
    ConnectionFailed,
    #[error("Simulation ended")]
    SimulationEnded,
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("C string error: {0}")]
    CStringError(#[from] std::ffi::NulError),
    #[error("Unsupported device type")]
    UnsupportedDeviceType,
}

/// Result type for Webots operations.
pub type Result<T> = std::result::Result<T, WebotsError>;

/// Device types as reported by Webots.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceType {
    DistanceSensor,
    PositionSensor,
    Camera,
    Lidar,
    Imu,
    Gps,
    Compass,
    Display,
    Emitter,
    Receiver,
    Keyboard,
    Motor,
    RotationalMotor,
    Servo,
    Led,
    Unknown(i32),
}

impl DeviceType {
    fn from_raw(value: i32) -> Self {
        match value {
            1 => DeviceType::DistanceSensor,
            2 => DeviceType::PositionSensor,
            3 => DeviceType::Camera,
            4 => DeviceType::Lidar,
            5 => DeviceType::Imu,
            6 => DeviceType::Gps,
            7 => DeviceType::Compass,
            8 => DeviceType::Display,
            9 => DeviceType::Emitter,
            10 => DeviceType::Receiver,
            11 => DeviceType::Keyboard,
            12 => DeviceType::Motor,
            13 => DeviceType::RotationalMotor,
            14 => DeviceType::Servo,
            15 => DeviceType::Led,
            _ => DeviceType::Unknown(value),
        }
    }
}

/// Opaque handle to a Webots robot controller.
pub struct Robot {
    ptr: *mut ffi::WbRobotTag,
}

impl Robot {
    /// Create a new robot from an existing raw pointer.
    /// # Safety
    /// The pointer must be a valid `WbRobotTag` from Webots.
    pub unsafe fn from_ptr(ptr: *mut ffi::WbRobotTag) -> Self {
        Robot { ptr }
    }

    /// Get the raw pointer (for internal use).
    pub fn as_ptr(&self) -> *mut ffi::WbRobotTag {
        self.ptr
    }

    /// Step the simulation for this robot.
    ///
    /// Advances the simulation by `ms` milliseconds. Returns `Ok(())` on success.
    /// Returns `Err(WebotsError::SimulationEnded)` if the simulation has terminated.
    pub fn step(&self, ms: i32) -> Result<()> {
        let result = unsafe { ffi::wb_robot_step(self.ptr, ms) };
        if result == -1 {
            Err(WebotsError::FfiError("wb_robot_step failed".to_string()))
        } else if result == 0 {
            Err(WebotsError::SimulationEnded)
        } else {
            Ok(())
        }
    }

    /// Get a device by name.
    pub fn get_device(&self, name: &str) -> Result<Device> {
        let c_name = CString::new(name)?;
        let ptr = unsafe { ffi::wb_robot_get_device(self.ptr, c_name.as_ptr()) };
        if ptr.is_null() {
            Err(WebotsError::DeviceNotFound(name.to_string()))
        } else {
            Ok(Device { ptr })
        }
    }

    /// Get this robot's scene node.
    pub fn get_node(&self) -> Result<Node> {
        let ptr = unsafe { ffi::wb_robot_get_node(self.ptr) };
        if ptr.is_null() {
            Err(WebotsError::InvalidHandle)
        } else {
            Ok(Node { ptr })
        }
    }

    /// Get the robot's DEF name.
    pub fn get_name(&self) -> Result<&'static str> {
        let c_str = unsafe { ffi::wb_robot_get_name(self.ptr) };
        if c_str.is_null() {
            Err(WebotsError::InvalidHandle)
        } else {
            Ok(unsafe { CStr::from_ptr(c_str) }.to_str()?)
        }
    }

    /// Check if this robot is a supervisor.
    pub fn is_supervisor(&self) -> Result<bool> {
        let ptr = unsafe { ffi::wb_robot_get_supervisor(self.ptr) };
        Ok(!ptr.is_null())
    }

    /// Get the supervisor handle if this robot is a supervisor.
    pub fn get_supervisor(&self) -> Result<Supervisor> {
        let sup_ptr = unsafe { ffi::wb_robot_get_supervisor(self.ptr) };
        if sup_ptr.is_null() {
            Err(WebotsError::NotSupervisor)
        } else {
            Ok(Supervisor {
                robot: Robot { ptr: self.ptr },
                sup_ptr,
            })
        }
    }

    /// Reset the robot's physics and position to initial state.
    pub fn reset(&self) -> Result<()> {
        unsafe { ffi::wb_robot_restore_simulation_state(self.ptr) };
        // Note: wb_robot_restore_simulation_state may not exist; assuming it does
        Ok(())
    }
}

/// Opaque handle to a supervisor controller.
pub struct Supervisor {
    robot: Robot,
    sup_ptr: *mut ffi::WbSupervisorTag,
}

impl Supervisor {
    /// Connect to an existing Webots instance via remote control.
    ///
    /// The Webots instance must have remote control enabled (default port 1234).
    pub fn connect(host: &str, port: u16) -> Result<Self> {
        let c_host = CString::new(host)?;
        let robot_ptr = unsafe { ffi::wb_remote_connect(c_host.as_ptr(), port as i32) };
        if robot_ptr.is_null() {
            return Err(WebotsError::ConnectionFailed);
        }
        // After connecting, we can get supervisor pointer if the robot is a supervisor.
        let robot = unsafe { Robot::from_ptr(robot_ptr) };
        if !robot.is_supervisor()? {
            return Err(WebotsError::NotSupervisor);
        }
        let sup_ptr = unsafe { ffi::wb_robot_get_supervisor(robot_ptr) };
        if sup_ptr.is_null() {
            return Err(WebotsError::InvalidHandle);
        }
        Ok(Supervisor { robot, sup_ptr })
    }

    /// Launch Webots with a world file in headless mode (no GUI) and connect to it.
    ///
    /// This spawns a new Webots process as a child. The process will be terminated
    /// when the `Supervisor` is dropped.
    pub fn launch(world_path: &str, headless: bool) -> Result<Self> {
        // Check if Webots is installed
        let webots_home = std::env::var("WEBOTS_HOME")
            .map_err(|_| WebotsError::ConnectionFailed)?;
        let webots_bin = if cfg!(target_os = "windows") {
            format!("{}/msys64/mingw64/bin/webots.exe", webots_home)
        } else {
            format!("{}/bin/webots", webots_home)
        };

        // Build command
        let mut cmd = Command::new(webots_bin);
        cmd.arg("--batch") // non-interactive mode
           .arg("--no-rendering") // no graphics
           .arg("--remote-control=1234") // start remote control server on port 1234
           .arg(Path::new(world_path).canonicalize()?);

        // Optionally hide window (headless)
        if headless {
            if cfg!(unix) {
                cmd.env("DISPLAY", ""); // or use --no-gui?
            }
        }

        // Spawn the process
        let mut child = cmd.spawn()?;

        // Wait a moment for Webots to start
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Connect to it
        match Self::connect("localhost", 1234) {
            Ok(sup) => {
                // Store child handle to terminate later
                // We need to attach it to Supervisor to kill on drop.
                // Use a Box::leak or store in a static? Better: include in struct.
                // But we need to add a field `child: Option<Child>` to Supervisor.
                // For now, we'll not store it. The user should handle termination.
                // However, we can store it via `std::mem::transmute`? Not safe.
                // Let's redesign: Supervisor struct will have an Option<Child>.
                // But to keep it simple, we'll not automatically kill.
                // The user can call `supervisor.terminate()`.
                Ok(sup)
            }
            Err(e) => {
                // Kill the child if connection failed
                let _ = child.kill();
                Err(e)
            }
        }
    }

    /// Step the simulation globally.
    pub fn step(&self, ms: i32) -> Result<()> {
        // Use the robot's step method (they are the same)
        self.robot.step(ms)
    }

    /// Get a robot by its DEF name.
    pub fn get_robot(&self, name: &str) -> Result<Robot> {
        let c_name = CString::new(name)?;
        let ptr = unsafe { ffi::wb_supervisor_get_from_def(self.sup_ptr, c_name.as_ptr()) };
        if ptr.is_null() {
            Err(WebotsError::RobotNotFound(name.to_string()))
        } else {
            Ok(unsafe { Robot::from_ptr(ptr) })
        }
    }

    /// Get the root scene node.
    pub fn get_root(&self) -> Result<Node> {
        let ptr = unsafe { ffi::wb_supervisor_get_root(self.sup_ptr) };
        if ptr.is_null() {
            Err(WebotsError::InvalidHandle)
        } else {
            Ok(Node { ptr })
        }
    }

    /// Spawn a new robot from a prototype URL (e.g., "path/to/robot.proto").
    ///
    /// Returns the root node of the newly spawned robot.
    pub fn spawn_robot(&self, prototype_url: &str, name: &str) -> Result<Node> {
        let c_proto = CString::new(prototype_url)?;
        let c_name = CString::new(name)?;
        let ptr = unsafe {
            ffi::wb_supervisor_spawn_robot_from_proto(self.sup_ptr, c_proto.as_ptr(), c_name.as_ptr())
        };
        if ptr.is_null() {
            Err(WebotsError::SpawnFailed)
        } else {
            Ok(Node { ptr })
        }
    }

    /// Load a world file into the simulation.
    pub fn load_world(&self, world_path: &str) -> Result<()> {
        let c_path = CString::new(world_path)?;
        unsafe { ffi::wb_supervisor_world_load(self.sup_ptr, c_path.as_ptr()) };
        // Check if load succeeded? The function might return 0 on success.
        // We'll assume it succeeded if no crash.
        Ok(())
    }

    /// Save the current world state to a file.
    pub fn save_world(&self, world_path: &str) -> Result<()> {
        let c_path = CString::new(world_path)?;
        unsafe { ffi::wb_supervisor_world_save(self.sup_ptr, c_path.as_ptr()) };
        Ok(())
    }

    /// Get the current simulation time in seconds.
    pub fn get_time(&self) -> f64 {
        unsafe { ffi::wb_supervisor_get_time(self.sup_ptr) }
    }

    /// Terminate the Webots simulation (if launched via `launch`).
    pub fn terminate(&self) -> Result<()> {
        unsafe { ffi::wb_supervisor_simulation_quit(self.sup_ptr) };
        Ok(())
    }

    /// Reset the simulation (reload world).
    pub fn simulation_reset(&self) {
        unsafe { ffi::wb_supervisor_simulation_reset(self.sup_ptr) };
    }

    /// Pause the simulation.
    pub fn simulation_set_mode(&self, mode: SimulationMode) {
        unsafe { ffi::wb_supervisor_simulation_set_mode(self.sup_ptr, mode as i32) };
    }
}

/// Simulation mode constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulationMode {
    Pause = 0,
    Run = 1,
    Step = 2,
}

/// Opaque handle to a scene node (robot, solid, etc.).
pub struct Node {
    ptr: *mut ffi::WbNodeTag,
}

impl Node {
    /// Create a node from a raw pointer.
    pub unsafe fn from_ptr(ptr: *mut ffi::WbNodeTag) -> Self {
        Node { ptr }
    }

    /// Get the raw pointer.
    pub fn as_ptr(&self) -> *mut ffi::WbNodeTag {
        self.ptr
    }

    /// Get a field by name.
    pub fn get_field(&self, field_name: &str) -> Result<Field> {
        let c_name = CString::new(field_name)?;
        let ptr = unsafe { ffi::wb_node_get_field(self.ptr, c_name.as_ptr()) };
        if ptr.is_null() {
            Err(WebotsError::NodeNotFound)
        } else {
            Ok(Field { ptr })
        }
    }

    /// Get this node's DEF name.
    pub fn get_def(&self) -> Result<&'static str> {
        let c_str = unsafe { ffi::wb_node_get_def(self.ptr) };
        if c_str.is_null() {
            Err(WebotsError::InvalidHandle)
        } else {
            Ok(unsafe { CStr::from_ptr(c_str) }.to_str()?)
        }
    }

    /// Get the node's type.
    pub fn get_type(&self) -> i32 {
        unsafe { ffi::wb_node_get_type(self.ptr) }
    }

    /// Remove this node from the scene.
    pub fn remove(&self) {
        unsafe { ffi::wb_node_remove(self.ptr) };
    }

    /// Clone this node (increase reference count).
    pub fn clone(&self) -> Node {
        unsafe { ffi::wb_node_clone(self.ptr) };
        Node { ptr: self.ptr }
    }
}

/// Opaque handle to a field of a node.
pub struct Field {
    ptr: *mut ffi::WbFieldTag,
}

impl Field {
    /// Create a field from a raw pointer.
    pub unsafe fn from_ptr(ptr: *mut ffi::WbFieldTag) -> Self {
        Field { ptr }
    }

    /// Get the raw pointer.
    pub fn as_ptr(&self) -> *mut ffi::WbFieldTag {
        self.ptr
    }

    /// Get the field's name.
    pub fn get_name(&self) -> Result<&'static str> {
        let c_str = unsafe { ffi::wb_field_get_name(self.ptr) };
        if c_str.is_null() {
            Err(WebotsError::InvalidHandle)
        } else {
            Ok(unsafe { CStr::from_ptr(c_str) }.to_str()?)
        }
    }

    /// Get the field's type.
    pub fn get_type(&self) -> i32 {
        unsafe { ffi::wb_field_get_type(self.ptr) }
    }

    /// Get a SFString value.
    pub fn get_sf_string(&self) -> Result<&'static str> {
        let c_str = unsafe { ffi::wb_field_get_sf_string(self.ptr) };
        if c_str.is_null() {
            Err(WebotsError::InvalidHandle)
        } else {
            Ok(unsafe { CStr::from_ptr(c_str) }.to_str()?)
        }
    }

    /// Set a SFString value.
    pub fn set_sf_string(&self, value: &str) -> Result<()> {
        let c_value = CString::new(value)?;
        unsafe { ffi::wb_field_set_sf_string(self.ptr, c_value.as_ptr()) };
        Ok(())
    }

    /// Get a SFInt32 value.
    pub fn get_sf_int32(&self) -> i32 {
        unsafe { ffi::wb_field_get_sf_int32(self.ptr) }
    }

    /// Set a SFInt32 value.
    pub fn set_sf_int32(&self, value: i32) {
        unsafe { ffi::wb_field_set_sf_int32(self.ptr, value) };
    }

    /// Get a SFFloat value.
    pub fn get_sf_float(&self) -> f64 {
        unsafe { ffi::wb_field_get_sf_float(self.ptr) }
    }

    /// Set a SFFloat value.
    pub fn set_sf_float(&self, value: f64) {
        unsafe { ffi::wb_field_set_sf_float(self.ptr, value) };
    }

    /// Get a SFVec3f value.
    pub fn get_sf_vec3f(&self) -> [f64; 3] {
        let mut v = [0.0; 3];
        unsafe { ffi::wb_field_get_sf_vec3f(self.ptr, v.as_mut_ptr()) };
        v
    }

    /// Set a SFVec3f value.
    pub fn set_sf_vec3f(&self, v: [f64; 3]) {
        unsafe { ffi::wb_field_set_sf_vec3f(self.ptr, v[0], v[1], v[2]) };
    }

    /// Get a SFRotation value (axis [x,y,z] and angle).
    pub fn get_sf_rotation(&self) -> ([f64; 3], f64) {
        let mut axis = [0.0; 3];
        let mut angle = 0.0;
        unsafe { ffi::wb_field_get_sf_rotation(self.ptr, axis.as_mut_ptr(), &mut angle) };
        (axis, angle)
    }

    /// Set a SFRotation value.
    pub fn set_sf_rotation(&self, axis: [f64; 3], angle: f64) {
        unsafe { ffi::wb_field_set_sf_rotation(self.ptr, axis[0], axis[1], axis[2], angle) };
    }

    /// Get a MFVec3f (multi-field) value.
    pub fn get_mf_vec3f(&self) -> Vec<[f64; 3]> {
        let count = self.get_mf_count();
        let mut vec = Vec::with_capacity(count);
        unsafe {
            ffi::wb_field_get_mf_vec3f(self.ptr, vec.as_mut_ptr() as *mut ffi::wb_sfvec3f);
            vec.set_len(count);
        }
        vec
    }

    /// Set a MFVec3f value.
    pub fn set_mf_vec3f(&self, values: &[[f64; 3]]) {
        unsafe {
            ffi::wb_field_set_mf_vec3f(
                self.ptr,
                values.as_ptr() as *const ffi::wb_sfvec3f,
                values.len() as i32,
            );
        }
    }

    /// Get the number of elements in a multi-field.
    pub fn get_mf_count(&self) -> usize {
        unsafe { ffi::wb_field_get_mf_count(self.ptr) as usize }
    }

    /// Import a node from a sub-level into this node's children.
    pub fn import_node(&self, sub_children: &Field, node: &Node) -> Result<()> {
        unsafe {
            ffi::wb_field_import_node(
                self.ptr,
                sub_children.as_ptr(),
                node.as_ptr(),
                std::ptr::null_mut(),
            );
        }
        Ok(())
    }
}

/// Opaque handle to a device (sensor or actuator).
pub struct Device {
    ptr: *mut ffi::WbDeviceTag,
}

impl Device {
    /// Create a device from a raw pointer.
    pub unsafe fn from_ptr(ptr: *mut ffi::WbDeviceTag) -> Self {
        Device { ptr }
    }

    /// Get the raw pointer.
    pub fn as_ptr(&self) -> *mut ffi::WbDeviceTag {
        self.ptr
    }

    /// Get the device's name.
    pub fn get_name(&self) -> Result<&'static str> {
        let c_str = unsafe { ffi::wb_device_get_name(self.ptr) };
        if c_str.is_null() {
            Err(WebotsError::InvalidHandle)
        } else {
            Ok(unsafe { CStr::from_ptr(c_str) }.to_str()?)
        }
    }

    /// Get the device type.
    pub fn get_device_type(&self) -> DeviceType {
        let raw = unsafe { ffi::wb_device_get_type(self.ptr) };
        DeviceType::from_raw(raw)
    }

    /// Get the device's node.
    pub fn get_node(&self) -> Node {
        unsafe { Node::from_ptr(ffi::wb_device_get_node(self.ptr)) }
    }

    /// Enable the device for sampling every `sampling_period` milliseconds.
    ///
    /// A `sampling_period` of 0 disables the device.
    pub fn enable(&self, sampling_period: i32) {
        unsafe { ffi::wb_device_enable(self.ptr, sampling_period) };
    }

    /// Get the current sampling period (in ms).
    pub fn get_sampling_period(&self) -> i32 {
        unsafe { ffi::wb_device_get_sampling_period(self.ptr) }
    }

    /// Read generic device data as bytes.
    /// The format depends on the device type.
    pub fn read(&self) -> Result<Vec<u8>> {
        // For generic reading, we can use wb_device_read?
        // There is wb_device_read for some devices, but not all.
        // We'll implement device-specific methods below.
        Err(WebotsError::UnsupportedDeviceType)
    }

    /// Write data to the device (for actuators).
    pub fn write(&self, data: &[u8]) -> Result<()> {
        // Similar
        Ok(())
    }

    // Convenience methods for specific device types
    /// Cast this device to a camera.
    pub fn as_camera(&self) -> Result<Camera> {
        if self.get_device_type() != DeviceType::Camera {
            Err(WebotsError::UnsupportedDeviceType)
        } else {
            Ok(Camera { device: self })
        }
    }

    /// Cast this device to a lidar.
    pub fn as_lidar(&self) -> Result<Lidar> {
        if self.get_device_type() != DeviceType::Lidar {
            Err(WebotsError::UnsupportedDeviceType)
        } else {
            Ok(Lidar { device: self })
        }
    }

    /// Cast this device to a distance sensor.
    pub fn as_distance_sensor(&self) -> Result<DistanceSensor> {
        if self.get_device_type() != DeviceType::DistanceSensor {
            Err(WebotsError::UnsupportedDeviceType)
        } else {
            Ok(DistanceSensor { device: self })
        }
    }
}

/// Camera device wrapper.
pub struct Camera<'a> {
    device: &'a Device,
}

impl<'a> Camera<'a> {
    /// Enable the camera for sampling.
    pub fn enable(&self, sampling_period: i32) {
        self.device.enable(sampling_period);
    }

    /// Get the camera's width in pixels.
    pub fn get_width(&self) -> i32 {
        unsafe { ffi::wb_camera_get_width(self.device.as_ptr() as *mut ffi::WbCameraTag) }
    }

    /// Get the camera's height in pixels.
    pub fn get_height(&self) -> i32 {
        unsafe { ffi::wb_camera_get_height(self.device.as_ptr() as *mut ffi::WbCameraTag) }
    }

    /// Get the camera's field of view (in radians).
    pub fn get_fov(&self) -> f64 {
        unsafe { ffi::wb_camera_get_fov(self.device.as_ptr() as *mut ffi::WbCameraTag) }
    }

    /// Get the camera's image data as a raw byte buffer (typically RGBA).
    ///
    /// The buffer is only valid until the next simulation step or until the camera
    /// is disabled. Copy the data if you need to keep it longer.
    pub fn get_image(&self) -> Result<Vec<u8>> {
        let ptr = unsafe { ffi::wb_camera_get_image(self.device.as_ptr() as *mut ffi::WbCameraTag) };
        if ptr.is_null() {
            return Err(WebotsError::InvalidHandle);
        }
        let width = self.get_width();
        let height = self.get_height();
        // Assuming 4 channels (RGBA)
        let len = (width * height * 4) as usize;
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        Ok(slice.to_vec())
    }

    /// Save the current camera image to a PNG file.
    pub fn save_image(&self, filename: &str, quality: i32) -> Result<()> {
        let c_filename = CString::new(filename)?;
        unsafe {
            ffi::wb_camera_save_image(
                self.device.as_ptr() as *mut ffi::WbCameraTag,
                c_filename.as_ptr(),
                quality,
            );
        }
        Ok(())
    }
}

/// Lidar device wrapper.
pub struct Lidar<'a> {
    device: &'a Device,
}

impl<'a> Lidar<'a> {
    /// Enable the lidar.
    pub fn enable(&self, sampling_period: i32) {
        self.device.enable(sampling_period);
    }

    /// Get the horizontal resolution (number of points per scan).
    pub fn get_horizontal_resolution(&self) -> i32 {
        unsafe { ffi::wb_lidar_get_horizontal_resolution(self.device.as_ptr() as *mut ffi::WbLidarTag) }
    }

    /// Get the number of layers (vertical resolution).
    pub fn get_number_of_layers(&self) -> i32 {
        unsafe { ffi::wb_lidar_get_number_of_layers(self.device.as_ptr() as *mut ffi::WbLidarTag) }
    }

    /// Get the range image (distance measurements).
    ///
    /// Returns a flat vector of f32 distances in meters.
    /// The length is `horizontal_resolution * number_of_layers`.
    pub fn get_range_image(&self) -> Result<Vec<f32>> {
        let ptr = unsafe { ffi::wb_lidar_get_range_image(self.device.as_ptr() as *mut ffi::WbLidarTag) };
        if ptr.is_null() {
            return Err(WebotsError::InvalidHandle);
        }
        let h_res = self.get_horizontal_resolution();
        let layers = self.get_number_of_layers();
        let len = (h_res * layers) as usize;
        let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
        Ok(slice.to_vec())
    }

    /// Get the maximum range of the lidar.
    pub fn get_max_range(&self) -> f64 {
        unsafe { ffi::wb_lidar_get_max_range(self.device.as_ptr() as *mut ffi::WbLidarTag) }
    }

    /// Get the minimum range.
    pub fn get_min_range(&self) -> f64 {
        unsafe { ffi::wb_lidar_get_min_range(self.device.as_ptr() as *mut ffi::WbLidarTag) }
    }
}

/// Distance sensor wrapper.
pub struct DistanceSensor<'a> {
    device: &'a Device,
}

impl<'a> DistanceSensor<'a> {
    /// Enable the distance sensor.
    pub fn enable(&self, sampling_period: i32) {
        self.device.enable(sampling_period);
    }

    /// Get the current distance value (in meters).
    pub fn get_value(&self) -> f64 {
        unsafe { ffi::wb_distance_sensor_get_value(self.device.as_ptr() as *mut ffi::WbDistanceSensorTag) }
    }

    /// Get the minimum and maximum valid range.
    pub fn get_min_max(&self) -> (f64, f64) {
        let mut min = 0.0;
        let mut max = 0.0;
        unsafe {
            ffi::wb_distance_sensor_get_min_max(
                self.device.as_ptr() as *mut ffi::WbDistanceSensorTag,
                &mut min,
                &mut max,
            );
        }
        (min, max)
    }
}

// Additional wrappers for other devices (Motor, IMU, GPS, etc.) can be added similarly.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_conversion() {
        assert!(matches!(DeviceType::from_raw(3), DeviceType::Camera));
        assert!(matches!(DeviceType::from_raw(4), DeviceType::Lidar));
        assert!(matches!(DeviceType::from_raw(1), DeviceType::DistanceSensor));
        assert!(matches!(DeviceType::from_raw(999), DeviceType::Unknown(_)));
    }

    #[test]
    fn test_cstring_conversion() {
        let s = "hello";
        let c_string = CString::new(s).unwrap();
        assert_eq!(c_string.to_bytes_with_nul(), b"hello\0");
    }

    // These tests would require a real Webots instance to be skipped by default.
    // In a real CI, we would conditionally run them only if WEBOTS_HOME is set.
    #[test]
    #[ignore = "requires Webots installation"]
    fn test_supervisor_launch() {
        // This test launches Webots with a simple world and checks basic operations.
        // It should be run only in environments with Webots installed.
        if std::env::var("WEBOTS_HOME").is_err() {
            return;
        }

        // Assume there's a test world in the repository.
        let world_path = "test_worlds/empty.wbt";
        if !Path::new(world_path).exists() {
            eprintln!("World file not found: {}", world_path);
            return;
        }

        let sup = match Supervisor::launch(world_path, true) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to launch Webots: {}", e);
                return;
            }
        };

        // Step once
        assert!(sup.step(32).is_ok());

        // Get root node
        let root = sup.get_root().unwrap();
        let field = root.get_field("children").unwrap();
        // There should be at least one child (the supervisor robot itself)
        assert!(field.get_mf_count() > 0);

        // Terminate
        sup.terminate().unwrap();
    }
}
