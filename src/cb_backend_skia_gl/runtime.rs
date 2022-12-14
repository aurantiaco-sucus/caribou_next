use std::cell::RefCell;
use std::convert::Into;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicBool;

use gl::types::*;
use glutin::{
    dpi::Position,
    window::{Window, WindowBuilder},
    event_loop::{ControlFlow, EventLoop},
    ContextWrapper,
    GlProfile,
    PossiblyCurrent,
    event::{
        Event,
        KeyboardInput,
        Ime,
        ElementState,
        WindowEvent
    },
};
use glutin::event_loop::ControlFlow::WaitUntil;
use log::{info};
use skia_safe::{
    gpu::gl::{Format, FramebufferInfo},
    gpu::{BackendRenderTarget, DirectContext, SurfaceOrigin},
    Color,
    ColorType,
    Surface
};
use crate::{
    caribou::{
        async_runtime,
        input::{Key}
    },
    cb_backend_skia_gl::{
        batch::skia_render_batch,
        input::gl_virtual_to_key
    }
};
use crate::caribou::math::ScalarPair;
use crate::cb_backend_skia_gl::input::{gl_mouse_button_interpret};

type WindowedContext = ContextWrapper<PossiblyCurrent, Window>;

type CbWindow = crate::caribou::window::Window;

pub struct SkGLEnv2 {
    pub surface: RefCell<Surface>,
    pub gr_context: RefCell<DirectContext>,
    pub windowed_context: WindowedContext,
    pub need_redraw: AtomicBool,
}

unsafe impl Send for SkGLEnv2 {}
unsafe impl Sync for SkGLEnv2 {}

pub static ENV_REGISTRY: RwLock<Vec<Arc<SkGLEnv2>>> = RwLock::new(Vec::new());

static mut SCALE_FACTOR: f32 = 1.0;

pub fn skia_set_scale_factor(factor: f32) {
    unsafe { SCALE_FACTOR = factor; }
}

pub fn skia_get_scale_factor() -> f32 {
    unsafe { SCALE_FACTOR }
}

fn skia_gl_create_surface(
    windowed_context: &WindowedContext,
    fb_info: &FramebufferInfo,
    gr_context: &mut DirectContext,
) -> Surface {
    let pixel_format = windowed_context.get_pixel_format();
    let size = windowed_context.window().inner_size();
    let backend_render_target = BackendRenderTarget::new_gl(
        (
            size.width.try_into().unwrap(),
            size.height.try_into().unwrap(),
        ),
        pixel_format.multisampling.map(|s| s.try_into().unwrap()),
        pixel_format.stencil_bits.try_into().unwrap(),
        *fb_info,
    );
    Surface::from_backend_render_target(
        gr_context,
        &backend_render_target,
        SurfaceOrigin::BottomLeft,
        ColorType::RGBA8888,
        None,
        None,
    )
        .unwrap()
}

pub fn skia_gl_launch(window: CbWindow, env_id: usize) {
    info!("Launching Skia GL window");
    let el = EventLoop::new();
    let wb = WindowBuilder::new().with_title("Caribou");

    let cb = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_stencil_buffer(8)
        .with_pixel_format(24, 8)
        .with_gl_profile(GlProfile::Core);
    #[cfg(not(feature = "wayland"))]
        let cb = cb
        .with_double_buffer(Some(true));

    let windowed_context = cb.build_windowed(wb, &el).unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };
    let pixel_format = windowed_context.get_pixel_format();

    info!("Pixel format: {:?}", pixel_format);

    gl::load_with(|s| windowed_context.get_proc_address(s));

    let mut gr_context = DirectContext::new_gl(None, None).unwrap();

    let fb_info = {
        let mut fboid: GLint = 0;
        unsafe { gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut fboid) };

        FramebufferInfo {
            fboid: fboid.try_into().unwrap(),
            format: Format::RGBA8.into(),
        }
    };

    windowed_context.window();
    info!("Creating Skia surface");
    let surface = skia_gl_create_surface(&windowed_context, &fb_info, &mut gr_context);
    skia_set_scale_factor(windowed_context.window().scale_factor() as f32);
    info!("Scale factor: {}", skia_get_scale_factor());

    windowed_context.window().set_ime_allowed(true);

    let mut frame = 0;

    // Guarantee the drop order inside the FnMut closure. `WindowedContext` _must_ be dropped after
    // `DirectContext`.
    //
    // https://github.com/rust-skia/rust-skia/issues/476
    ENV_REGISTRY.write().unwrap().push(Arc::new(SkGLEnv2 {
        surface: surface.into(),
        gr_context: gr_context.into(),
        windowed_context,
        need_redraw: AtomicBool::new(true),
    }));

    let mut mouse_pos: ScalarPair = (0.0, 0.0).into();
    let mut ret_vec: Vec<Key> = Vec::new();

    // TODO: Get FPS of monitor and use that as the max FPS
    let maximum_wait = std::time::Duration::from_millis(1000 / 60);

    info!("Launching event loop");
    el.run(move |event, _, control_flow| {
        *control_flow = WaitUntil(std::time::Instant::now() + std::time::Duration::from_millis(16));

        #[allow(deprecated)]
        match event {
            Event::LoopDestroyed => {}
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    let env = ENV_REGISTRY.read().unwrap()[env_id].clone();
                    let mut surface = env.surface.borrow_mut();
                    *surface = skia_gl_create_surface(
                        &env.windowed_context, &fb_info, &mut env.gr_context.borrow_mut());
                    drop(surface);
                    env.windowed_context.resize(physical_size);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                    KeyboardInput {
                        scancode,
                        virtual_keycode,
                        modifiers,
                        ..
                    },
                    ..
                } => {
                    let window_clone = window.clone();
                    if let Some(vir) = virtual_keycode {
                        let key = gl_virtual_to_key(vir);
                        if ret_vec.contains(&key) {
                            ret_vec.retain(|x| *x != key);
                            async_runtime().spawn(async move {
                                let window = window_clone;
                                window.key_down.remove(&key).await;
                            });
                        } else {
                            ret_vec.push(key);
                            async_runtime().spawn(async move {
                                let window = window_clone;
                                window.key_down.push(key).await;
                            });
                        }
                    }
                    frame += 1;
                }
                WindowEvent::CursorEntered { .. } => {
                    //let window_clone = window.clone();
                }
                WindowEvent::CursorLeft { .. } => {
                    let window_clone = window.clone();
                    async_runtime().spawn(async move {
                        let window = window_clone;
                        window.mouse_pos.take().await;
                    });
                }
                WindowEvent::CursorMoved {
                    position,
                    modifiers,
                    ..
                } => {
                    let new_pos: ScalarPair = (position.x as f32, position.y as f32).into();
                    let new_pos = new_pos.times(1_f32 / skia_get_scale_factor());
                    mouse_pos = new_pos;
                    let window_clone = window.clone();
                    async_runtime().spawn(async move {
                        let window = window_clone;
                        window.mouse_pos.put(mouse_pos).await;
                    });
                }
                WindowEvent::MouseInput {
                    state,
                    button,
                    modifiers,
                    ..
                } => {
                    let button = gl_mouse_button_interpret(button);
                    match state {
                        ElementState::Pressed => {
                            let window_clone = window.clone();
                            async_runtime().spawn(async move {
                                let window = window_clone;
                                window.mouse_down.push(button).await;
                            });
                        }
                        ElementState::Released => {
                            let window_clone = window.clone();
                            async_runtime().spawn(async move {
                                let window = window_clone;
                                window.mouse_down.remove(&button).await;
                            });
                        }
                    }
                }
                WindowEvent::Ime(ev) => match ev {
                    Ime::Enabled => {
                        println!("Ime enabled");
                    }
                    Ime::Preedit(pre, pos) => {
                        let env = ENV_REGISTRY.read().unwrap()[env_id].clone();
                        env.windowed_context.window()
                            .set_ime_position(Position::Logical((100.0, 100.0).into()));
                        println!("Ime preedit: {:?} {:?}", pre, pos);
                    }
                    Ime::Commit(str) => {
                        println!("Ime commit: {:?}", str);
                    }
                    Ime::Disabled => {}
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                let env = ENV_REGISTRY.read().unwrap()[env_id].clone();
                {
                    let mut surface = env.surface.borrow_mut();
                    let canvas = surface.canvas();
                    canvas.clear(Color::WHITE);
                    canvas.reset_matrix();
                    let scale_factor = skia_get_scale_factor();
                    canvas.scale((scale_factor, scale_factor));
                    canvas.save();
                    let batch = async_runtime().block_on(async {
                        window.root.get().await.batch.get_cloned().await
                    });
                    skia_render_batch(canvas, batch);
                    canvas.restore();
                    canvas.flush();
                }
                env.windowed_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}