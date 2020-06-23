use crate::render_types::OPENGL_TO_WGPU_MATRIX;
use winit::event::*;

pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    pub fn new(sc_width: u32, sc_height: u32) -> Self {
        Camera {
            eye: (0.0, 0.0, 5.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: (sc_width as f32) / (sc_height as f32),
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        }
    }
}

#[derive(Default)]
pub struct CameraController {
    speed: f32,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            ..CameraController::default()
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Q => {
                        self.is_up_pressed = pressed;
                        true
                    }
                    VirtualKeyCode::E => {
                        self.is_down_pressed = pressed;
                        true
                    }
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.is_forward_pressed = pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.is_left_pressed = pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update_camera(&mut self, camera: &mut Camera) {
        use cgmath::InnerSpace;

        let forward = (camera.target - camera.eye).normalize();
        let right = forward.cross(camera.up);

        if self.is_forward_pressed {
            camera.eye += forward * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward * self.speed;
        }
        if self.is_right_pressed {
            camera.eye += right * self.speed;
        }
        if self.is_left_pressed {
            camera.eye -= right * self.speed;
        }
    }
}
