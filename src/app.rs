use std::f32::consts::PI;
use std::sync::Arc;
use crate::{camera, colors, context, debug, input, maths, model, scene};
use cgmath::{Deg, Matrix, Matrix4, Point3, Quaternion, Rad, Rotation3, SquareMatrix, Vector3, Vector4};
use color_eyre::Result;
use std::time::{Duration, Instant};
use glium::{Surface, uniform};
use glium::uniforms::UniformBuffer;
use image::open;
use log::{debug, info};
use winit::keyboard::KeyCode;

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

use input::Input;
use scene::Scene;
use context::{OpenGLContext, RenderingContext};
use crate::model::{Model, ModelInstance, Transform};
use crate::vertex::Vertex;

pub struct App {
    input: Input,
    scene: Scene,
    opengl_context: OpenGLContext,
    rendering_context: RenderingContext,
    teapot: Arc<Model>
}

impl App {
    pub fn new(event_loop: &EventLoop<()>) -> Self {
        color_eyre::install().unwrap();
        debug::set_up_logging();

        // TODO deferred rendering https://learnopengl.com/Advanced-Lighting/Deferred-Shading
        let opengl_context = OpenGLContext::new("We glutin teapot now", event_loop).unwrap();
        let rendering_context = RenderingContext::new(
            "assets/shaders/default.vert",
            "assets/shaders/default.frag",
            &opengl_context.display
        ).unwrap();

        let mut scene = Scene::new(camera::Camera::new_fps(
            Point3::new(5.0, 2.0, 5.0),
            Vector3::new(0.0, 0.0, 1.0),
        ));

        let teapot = Model::load("assets/models/teapot.glb", &opengl_context.display).unwrap();

        scene.model_instances.reserve(100);

        let square_size = 10;

        for x in 0..square_size {
            for y in 0..square_size {
                scene.model_instances.push(ModelInstance {
                    model: teapot.clone(),
                    transform: Transform {
                        translation: Vector3::new(x as f32, y as f32, 0.0),
                        ..Transform::default()
                    },
                })
            }
        }

        let input = Input::new();

        Self {
            opengl_context,
            rendering_context,
            scene,
            input,
            teapot
        }
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        let mut frame_state = FrameState {
            start: Instant::now(),
            recreate_swapchain: false,
            frame_count: 0,
            deltatime: 0.0,
            fps: 0.0
        };

        event_loop
            .run(move |event, event_loop_window_target| {
                // println!("{:?}", event);
                event_loop_window_target.set_control_flow(ControlFlow::Poll);

                match event {
                    Event::WindowEvent {
                        event: window_event,
                        window_id,
                    } if window_id == self.opengl_context.window.id() => {
                        match window_event {
                            WindowEvent::CloseRequested => event_loop_window_target.exit(),
                            WindowEvent::KeyboardInput { event, .. } => {
                                self.input.process_key_event(event)
                            }
                            WindowEvent::Resized(new_size) => {
                                self.opengl_context.display.resize((new_size.width, new_size.height));

                                // Set current aspect ratio
                                self.scene.camera.set_aspect_ratio(new_size.width as f32 / new_size.height as f32);
                            }
                            WindowEvent::RedrawRequested => {
                                let start = Instant::now();

                                if self.input.key_pressed(KeyCode::Escape) {
                                    event_loop_window_target.exit();
                                }

                                self.scene.camera.update(&self.input);

                                self.render(&mut frame_state);

                                frame_state.frame_count = (frame_state.frame_count + 1) % u128::MAX;
                                self.input.reset_just_released();

                                frame_state.deltatime = start.elapsed().as_secs_f64();
                                frame_state.fps = (1.0 / frame_state.deltatime) as f32;

                                debug!("{}", frame_state.fps);
                            }
                            _ => (),
                        }
                    }
                    Event::AboutToWait => self.opengl_context.window.request_redraw(),
                    _ => (),
                }
            })
            .unwrap();
    }

    fn render(&mut self, frame_state: &mut FrameState) {
        let window_size = self.opengl_context.window.inner_size();
        if window_size.width == 0 || window_size.height == 0 {
            return;
        }

        for model_instance in self.scene.model_instances.iter_mut() {
            model_instance.transform.rotation = Quaternion::from_angle_y(Deg((frame_state.frame_count % 360) as f32));
        }

        self.scene.render(
            &self.rendering_context.program,
            &self.opengl_context.display
        );
    }
}

struct FrameState {
    start: Instant,
    recreate_swapchain: bool,
    frame_count: u128,
    deltatime: f64,
    fps: f32,
}
