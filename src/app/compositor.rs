use std::{
    error::Error,
    os::unix::io::OwnedFd,
    sync::{Arc, Mutex},
    time::Instant, // Added import
};

use eframe::egui::Vec2;
use smithay::{
    backend::{
        input::KeyState,
        renderer::{
            element::{
                surface::{render_elements_from_surface_tree, WaylandSurfaceRenderElement},
                Kind,
            },
            utils::{draw_render_elements, on_commit_buffer_handler},
            Color32F, Frame, Renderer,
        },
    },
    delegate_compositor, delegate_data_device, delegate_seat, delegate_shm, delegate_xdg_shell,
    input::{
        self,
        keyboard::{FilterResult, KeyboardHandle},
        Seat, SeatHandler, SeatState,
    },
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{protocol::wl_seat, Display},
    },
    utils::{Rectangle, Serial, Size, Transform},
    wayland::{
        buffer::BufferHandler,
        compositor::{
            with_surface_tree_downward, CompositorClientState, CompositorHandler, CompositorState,
            SurfaceAttributes, TraversalAction,
        },
        selection::{
            data_device::{
                ClientDndGrabHandler, DataDeviceHandler, DataDeviceState, ServerDndGrabHandler,
            },
            SelectionHandler,
        },
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState,
        },
        shm::{ShmHandler, ShmState},
    },
};
use wayland_server::{
    backend::{ClientData, ClientId, DisconnectReason},
    protocol::{
        wl_buffer,
        wl_surface::{self, WlSurface},
    },
    Client, ListeningSocket,
};

use crate::utils::wayland::bind_socket;

use super::renderer::PolarBearRenderer;

pub struct PolarBearCompositor {
    state: State,
    display: Display<State>,
    listener: ListeningSocket,
    clients: Arc<Mutex<Vec<Client>>>,
    start_time: Instant,
    seat: Seat<State>,
    keyboard: KeyboardHandle<State>,
}

struct State {
    compositor_state: CompositorState,
    xdg_shell_state: XdgShellState,
    shm_state: ShmState,
    data_device_state: DataDeviceState,
    seat_state: SeatState<Self>,
}

impl BufferHandler for State {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Activated);
        });
        surface.send_configure();
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {
        // Handle popup creation here
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
        // Handle popup grab here
    }

    fn reposition_request(
        &mut self,
        _surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {
        // Handle popup reposition here
    }
}

impl SelectionHandler for State {
    type SelectionUserData = ();
}

impl DataDeviceHandler for State {
    fn data_device_state(&self) -> &DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for State {}
impl ServerDndGrabHandler for State {
    fn send(&mut self, _mime_type: String, _fd: OwnedFd, _seat: Seat<Self>) {}
}

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
    }
}

impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}
    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: input::pointer::CursorImageStatus) {}
}

pub fn send_frames_surface_tree(surface: &wl_surface::WlSurface, time: u32) {
    with_surface_tree_downward(
        surface,
        (),
        |_, _, &()| TraversalAction::DoChildren(()),
        |_surf, states, &()| {
            // the surface may not have any user_data if it is a subsurface and has not
            // yet been commited
            for callback in states
                .cached_state
                .get::<SurfaceAttributes>()
                .current()
                .frame_callbacks
                .drain(..)
            {
                callback.done(time);
            }
        },
        |_, _, &()| true,
    );
}

#[derive(Default)]
struct ClientState {
    compositor_state: CompositorClientState,
}
impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {
        println!("initialized");
    }

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {
        println!("disconnected");
    }
}

// Macros used to delegate protocol handling to types in the app state.
delegate_xdg_shell!(State);
delegate_compositor!(State);
delegate_shm!(State);
delegate_seat!(State);
delegate_data_device!(State);

impl PolarBearCompositor {
    pub fn build() -> Result<PolarBearCompositor, Box<dyn Error>> {
        let display = Display::new()?;
        let dh = display.handle();

        let mut seat_state = SeatState::new();
        let mut seat = seat_state.new_wl_seat(&dh, "Polar Bear");

        let listener = bind_socket()?;
        let clients = Arc::new(Mutex::new(Vec::new()));

        let start_time = Instant::now();

        let keyboard = seat.add_keyboard(Default::default(), 200, 200).unwrap();

        let state = State {
            compositor_state: CompositorState::new::<State>(&dh),
            xdg_shell_state: XdgShellState::new::<State>(&dh),
            shm_state: ShmState::new::<State>(&dh, vec![]),
            data_device_state: DataDeviceState::new::<State>(&dh),
            seat_state,
        };

        Ok(PolarBearCompositor {
            state,
            listener,
            clients,
            start_time,
            display,
            seat,
            keyboard,
        })
    }

    pub fn keyboard_input_handler(self, keycode: u32, key_state: KeyState) {
        let mut state = self.state;
        self.keyboard.input::<(), _>(
            &mut state,
            keycode.into(),
            key_state,
            0.into(),
            0,
            |_, _, _| {
                //
                FilterResult::Forward
            },
        );
    }

    pub fn pointer_motion_absolute_handler(self) {
        if let Some(surface) = self
            .state
            .xdg_shell_state
            .toplevel_surfaces()
            .iter()
            .next()
            .cloned()
        {
            let surface = surface.wl_surface().clone();
            let mut state = self.state;
            self.keyboard.set_focus(&mut state, Some(surface), 0.into());
        };
    }

    pub fn draw(
        &mut self,
        mut renderer: PolarBearRenderer,
        size: Vec2,
    ) -> Result<(), Box<dyn Error>> {
        let size = Size::from((size.x as i32, size.y as i32));

        let damage = Rectangle::from_size(size);

        let elements = self
            .state
            .xdg_shell_state
            .toplevel_surfaces()
            .iter()
            .flat_map(|surface| {
                render_elements_from_surface_tree(
                    &mut renderer,
                    surface.wl_surface(),
                    (0, 0),
                    1.0,
                    1.0,
                    Kind::Unspecified,
                )
            })
            .collect::<Vec<WaylandSurfaceRenderElement<PolarBearRenderer>>>();

        let mut frame = renderer.render(size, Transform::Flipped180).unwrap();
        frame
            .clear(Color32F::new(0.1, 0.0, 0.0, 1.0), &[damage])
            .unwrap();
        draw_render_elements(&mut frame, 1.0, &elements, &[damage]).unwrap();
        let _ = frame.finish().unwrap();

        for surface in self.state.xdg_shell_state.toplevel_surfaces() {
            send_frames_surface_tree(
                surface.wl_surface(),
                self.start_time.elapsed().as_millis() as u32,
            );
        }

        if let Some(stream) = self.listener.accept()? {
            println!("Got a client: {:?}", stream);

            let client = self
                .display
                .handle()
                .insert_client(stream, Arc::new(ClientState::default()))
                .unwrap();
            self.clients.lock().unwrap().push(client);
        }

        let state = &mut self.state;
        self.display.dispatch_clients(state)?;
        self.display.flush_clients()?;

        // let mut damage = match damage {
        //     Some(damage) if self.damage_tracking && !damage.is_empty() => {
        //         let bind_size = self
        //             .bind_size
        //             .expect("submitting without ever binding the renderer.");
        //         let damage = damage
        //             .iter()
        //             .map(|rect| {
        //                 Rectangle::new(
        //                     (rect.loc.x, bind_size.h - rect.loc.y - rect.size.h).into(),
        //                     rect.size,
        //                 )
        //             })
        //             .collect::<Vec<_>>();
        //         Some(damage)
        //     }
        //     _ => None,
        // };

        // // Request frame callback.
        // self.window.pre_present_notify();
        // self.egl_surface.swap_buffers(damage.as_deref_mut())?;
        Ok(())
    }
}
