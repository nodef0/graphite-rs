use futures::executor::block_on;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

// mod model;
mod camera;
mod const_mesh;
mod geometry;
mod pipelines;
mod render;
mod render_types;
mod texture;

use camera::{Camera, CameraController};
use const_mesh::{CIRCLE_INDICES, CIRCLE_VERTICES, PENTAGON_INDICES, PENTAGON_VERTICES};
use pipelines::pbr::{Pbr, PbrRenderPass, PbrState};
use pipelines::simple::{Simple, SimpleRenderPass, SimpleState};
use render::Graphics;

struct State {
    graphics: Graphics,
    simple: Simple,
    simple_state: SimpleState,
    pbr: Pbr,
    pbr_state: PbrState,
    camera: Camera,
    camera_controller: CameraController,
    model_angle: f32,
    model_speed: f32,
    size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    is_pbr: bool,
}

impl State {
    async fn new(window: &Window) -> Self {
        let tree_diffuse_bytes = include_bytes!("../res/happy-tree.png");
        let face_diffuse_bytes = include_bytes!("../res/face.jpg");

        let model_angle = 0.0;
        let model_speed = 0.02;
        let size = window.inner_size();

        let graphics = Graphics::new(window).await;
        let device = &graphics.device;
        let sc_desc = &graphics.sc_desc;

        let camera_controller = CameraController::new(0.2);
        let camera = Camera::new(sc_desc.width, sc_desc.height);

        let simple = Simple::new(&device, &sc_desc);
        let mut simple_state = SimpleState::new(&device, &simple, &camera);

        let _ = simple_state.add_texture(&device, &graphics.queue, &simple, tree_diffuse_bytes);
        let _ = simple_state.add_texture(&device, &graphics.queue, &simple, face_diffuse_bytes);

        let clear_color = wgpu::Color {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 1.0,
        };

        let _ = simple_state.add_geometry(&device, PENTAGON_VERTICES, PENTAGON_INDICES);
        let _ = simple_state.add_geometry(&device, CIRCLE_VERTICES, CIRCLE_INDICES);

        let pbr = Pbr::new(&device, &sc_desc);
        let pbr_state = PbrState::new(&device, &sc_desc, &graphics.queue, &pbr, &camera);
        let is_pbr = true;

        Self {
            graphics,
            simple,
            simple_state,
            pbr,
            pbr_state,
            camera,
            camera_controller,
            model_angle,
            model_speed,
            size,
            clear_color,
            is_pbr,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.graphics.resize(new_size);
        self.pbr_state
            .resize(&self.graphics.device, &self.graphics.sc_desc);
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        if self.camera_controller.process_events(event) {
            true
        } else {
            match event {
                WindowEvent::CursorMoved { position, .. } => {
                    self.clear_color.r = position.x as f64 / (self.size.width as f64);
                    self.clear_color.g = position.y as f64 / (self.size.height as f64);
                    return true;
                }
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        ..
                    } => {
                        if !self.is_pbr {
                            self.simple_state.inc_geometry_index();
                        }
                    }
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::T),
                        ..
                    } => {
                        if !self.is_pbr {
                            self.simple_state.inc_texture_index();
                        }
                    }
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::R),
                        ..
                    } => {
                        self.is_pbr = !self.is_pbr;
                    }
                    _ => {}
                },
                _ => {}
            }
            false
        }
    }

    fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        if self.is_pbr {
            self.pbr_state.mvp.update_view_proj(&self.camera);
        } else {
            self.model_angle += self.model_speed;
            self.simple_state.update_uniforms(
                &self.graphics.device,
                &self.graphics.queue,
                &self.camera,
                self.model_angle,
            );
        }
    }

    fn render(&mut self) {
        if self.is_pbr {
            self.graphics
                .render(&self.pbr.pipeline, &PbrRenderPass::new(&mut self.pbr_state));
        } else {
            self.graphics.render(
                &self.simple.pipeline,
                &SimpleRenderPass {
                    clear_color: self.clear_color,
                    state: &self.simple_state,
                },
            );
        }
    }
}

fn main() {
    // let _ = model::load_gltf("res/skin.gltf");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("Failed to build window");

    let mut state = block_on(State::new(&window));

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            state.update();
            state.render();
        }
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window.id() => {
            if !state.input(event) {
                match event {
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(**new_inner_size);
                    }
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => {
                        if let KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } = input
                        {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    });
}
