//! Common traits for input backends to receive input from.

use std::path::PathBuf;

pub use xkbcommon::xkb::Keycode;

mod tablet;

pub use tablet::{
    ProximityState, TabletToolAxisEvent, TabletToolButtonEvent, TabletToolCapabilities, TabletToolDescriptor,
    TabletToolEvent, TabletToolProximityEvent, TabletToolTipEvent, TabletToolTipState, TabletToolType,
};

#[cfg(feature = "wayland_frontend")]
use wayland_server::protocol::wl_pointer;

use crate::utils::{Logical, Point, Raw, Size};

/// Trait for generic functions every input device does provide
pub trait Device: PartialEq + Eq + std::hash::Hash {
    /// Unique id of a single device at a point in time.
    ///
    /// Note: This means ids may be re-used by the backend for later devices.
    fn id(&self) -> String;
    /// Human-readable name of the device
    fn name(&self) -> String;
    /// Test if this device has a specific capability
    fn has_capability(&self, capability: DeviceCapability) -> bool;

    /// Returns device USB (product,vendor) id
    fn usb_id(&self) -> Option<(u32, u32)>;

    /// Returns the syspath of the device.
    ///
    /// The path is an absolute path and includes the sys mount point.
    fn syspath(&self) -> Option<PathBuf>;
}

/// Set of input types a device may provide
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(missing_docs)] // self explainatory
pub enum DeviceCapability {
    Keyboard,
    Pointer,
    Touch,
    TabletTool,
    TabletPad,
    Gesture,
    Switch,
}

/// Trait for generic functions every input event does provide
pub trait Event<B: InputBackend> {
    /// Timestamp in milliseconds
    fn time_msec(&self) -> u32 {
        (self.time() / 1000) as u32
    }

    /// Timestamp in microseconds, with an undefined base.
    ///
    /// Libinput does not guarantee that timestamps always increase monotonically.
    // # TODO:
    // - check if events can even arrive out of order.
    // - Make stronger time guarantees, if possible
    fn time(&self) -> u64;

    /// Returns the device, that generated this event
    fn device(&self) -> B::Device;
}

/// Used to mark events never emitted by an [`InputBackend`] implementation.
///
/// Implements all event types and can be used in place for any [`Event`] type,
/// that is not used by an [`InputBackend`] implementation. Initialization is not
/// possible, making accidental use impossible and enabling a lot of possible
/// compiler optimizations.
#[derive(Debug)]
pub enum UnusedEvent {}

impl<B: InputBackend> Event<B> for UnusedEvent {
    fn time(&self) -> u64 {
        match *self {}
    }

    fn device(&self) -> B::Device {
        match *self {}
    }
}

/// State of key on a keyboard. Either pressed or released
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum KeyState {
    /// Key is released
    Released,
    /// Key is pressed
    Pressed,
}

/// Trait for keyboard event
pub trait KeyboardKeyEvent<B: InputBackend>: Event<B> {
    /// Returns the numerical button code of the keyboard button.
    ///
    /// The value will correspond to one `KEY_` constants from  the Linux [input event codes] inside
    /// `input-event-codes.h`.
    ///
    /// [input event codes]: https://gitlab.freedesktop.org/libinput/libinput/-/blob/main/include/linux/linux/input-event-codes.h
    fn key_code(&self) -> Keycode;

    /// State of the key
    fn state(&self) -> KeyState;

    /// Total number of keys pressed on all devices on the associated [`Seat`](crate::input::Seat)
    fn count(&self) -> u32;
}

impl<B: InputBackend> KeyboardKeyEvent<B> for UnusedEvent {
    fn key_code(&self) -> Keycode {
        match *self {}
    }

    fn state(&self) -> KeyState {
        match *self {}
    }

    fn count(&self) -> u32 {
        match *self {}
    }
}

/// A particular mouse button
#[non_exhaustive]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum MouseButton {
    /// Left mouse button
    Left,
    /// Middle mouse button
    Middle,
    /// Right mouse button
    Right,
    /// Back mouse button.
    Back,
    /// Forward mouse button.
    Forward,
}

/// State of a button on a pointer device, like mouse or tablet tool. Either pressed or released
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum ButtonState {
    /// Button is released
    Released,
    /// Button is pressed
    Pressed,
}

/// Common methods pointer event generated by pressed buttons do implement
pub trait PointerButtonEvent<B: InputBackend>: Event<B> {
    /// Pressed button of the event.
    ///
    /// This may return [`None`] if the button pressed in the event is not a standard mouse button. You may
    /// obtain the button code using [`PointerButtonEvent::button_code`].
    fn button(&self) -> Option<MouseButton> {
        // These values are coming from <linux/input-event-codes.h>.
        const BTN_LEFT: u32 = 0x110;
        const BTN_RIGHT: u32 = 0x111;
        const BTN_MIDDLE: u32 = 0x112;
        const BTN_SIDE: u32 = 0x113;
        const BTN_EXTRA: u32 = 0x114;
        const BTN_FORWARD: u32 = 0x115;
        const BTN_BACK: u32 = 0x116;

        match self.button_code() {
            BTN_LEFT => Some(MouseButton::Left),
            BTN_RIGHT => Some(MouseButton::Right),
            BTN_MIDDLE => Some(MouseButton::Middle),
            BTN_BACK | BTN_SIDE => Some(MouseButton::Back),
            BTN_FORWARD | BTN_EXTRA => Some(MouseButton::Forward),
            _ => None,
        }
    }

    /// Returns the numerical button code of the mouse button.
    ///
    /// The value will correspond to one `BTN_` constants from  the Linux [input event codes] inside
    /// `input-event-codes.h`.
    ///
    /// [input event codes]: https://gitlab.freedesktop.org/libinput/libinput/-/blob/main/include/linux/linux/input-event-codes.h
    fn button_code(&self) -> u32;

    /// State of the button
    fn state(&self) -> ButtonState;
}

impl<B: InputBackend> PointerButtonEvent<B> for UnusedEvent {
    fn button_code(&self) -> u32 {
        match *self {}
    }

    fn state(&self) -> ButtonState {
        match *self {}
    }
}
/// Trait for gesture begin events.
pub trait GestureBeginEvent<B: InputBackend>: Event<B> {
    /// Number of fingers.
    fn fingers(&self) -> u32;
}

impl<B: InputBackend> GestureBeginEvent<B> for UnusedEvent {
    fn fingers(&self) -> u32 {
        match *self {}
    }
}

/// Trait for gesture end events.
pub trait GestureEndEvent<B: InputBackend>: Event<B> {
    /// True if event was cancelled.
    fn cancelled(&self) -> bool;
}

impl<B: InputBackend> GestureEndEvent<B> for UnusedEvent {
    fn cancelled(&self) -> bool {
        match *self {}
    }
}

/// Trait for gesture swipe begin event.
pub trait GestureSwipeBeginEvent<B: InputBackend>: GestureBeginEvent<B> {}

impl<B: InputBackend> GestureSwipeBeginEvent<B> for UnusedEvent {}

/// Trait for gesture swipe update event.
pub trait GestureSwipeUpdateEvent<B: InputBackend>: Event<B> {
    /// Delta of center on the x axis from last begin/update.
    fn delta_x(&self) -> f64;

    /// Delta of center on the y axis from last begin/update.
    fn delta_y(&self) -> f64;

    /// Delta between the last and new gesture center.
    fn delta(&self) -> Point<f64, Logical> {
        (self.delta_x(), self.delta_y()).into()
    }
}

impl<B: InputBackend> GestureSwipeUpdateEvent<B> for UnusedEvent {
    fn delta_x(&self) -> f64 {
        match *self {}
    }

    fn delta_y(&self) -> f64 {
        match *self {}
    }
}

/// Trait for gesture swipe end event.
pub trait GestureSwipeEndEvent<B: InputBackend>: GestureEndEvent<B> {}

impl<B: InputBackend> GestureSwipeEndEvent<B> for UnusedEvent {}

/// Trait for gesture pinch begin event.
pub trait GesturePinchBeginEvent<B: InputBackend>: GestureBeginEvent<B> {}

impl<B: InputBackend> GesturePinchBeginEvent<B> for UnusedEvent {}

/// Trait for gesture pinch update event.
pub trait GesturePinchUpdateEvent<B: InputBackend>: Event<B> {
    /// Delta of center on the x axis from last begin/update.
    fn delta_x(&self) -> f64;

    /// Delta of center on the y axis from last begin/update.
    fn delta_y(&self) -> f64;

    /// Absolute scale compared to begin event.
    fn scale(&self) -> f64;

    /// Relative angle in degrees from last begin/update.
    fn rotation(&self) -> f64;

    /// Delta between the last and new gesture center.
    fn delta(&self) -> Point<f64, Logical> {
        (self.delta_x(), self.delta_y()).into()
    }
}

impl<B: InputBackend> GesturePinchUpdateEvent<B> for UnusedEvent {
    fn delta_x(&self) -> f64 {
        match *self {}
    }

    fn delta_y(&self) -> f64 {
        match *self {}
    }

    fn scale(&self) -> f64 {
        match *self {}
    }

    fn rotation(&self) -> f64 {
        match *self {}
    }
}

/// Trait for gesture pinch end event.
pub trait GesturePinchEndEvent<B: InputBackend>: GestureEndEvent<B> {}

impl<B: InputBackend> GesturePinchEndEvent<B> for UnusedEvent {}

/// Trait for gesture hold begin event.
pub trait GestureHoldBeginEvent<B: InputBackend>: GestureBeginEvent<B> {}

impl<B: InputBackend> GestureHoldBeginEvent<B> for UnusedEvent {}

/// Trait for gesture hold end event.
pub trait GestureHoldEndEvent<B: InputBackend>: GestureEndEvent<B> {}

impl<B: InputBackend> GestureHoldEndEvent<B> for UnusedEvent {}

/// Axis when scrolling
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Axis {
    /// Vertical axis
    Vertical,
    /// Horizontal axis
    Horizontal,
}

/// Source of an axis when scrolling
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AxisSource {
    /// Finger. Mostly used for trackpads.
    ///
    /// Guarantees that a scroll sequence is terminated with a scroll value of 0.
    /// A caller may use this information to decide on whether kinetic scrolling should
    /// be triggered on this scroll sequence.
    ///
    /// The coordinate system is identical to the
    /// cursor movement, i.e. a scroll value of 1 represents the equivalent relative
    /// motion of 1.
    Finger,
    /// Continuous scrolling device. Almost identical to [`Finger`](AxisSource::Finger)
    ///
    /// No terminating event is guaranteed (though it may happen).
    ///
    /// The coordinate system is identical to
    /// the cursor movement, i.e. a scroll value of 1 represents the equivalent relative
    /// motion of 1.
    Continuous,
    /// Scroll wheel.
    ///
    /// No terminating event is guaranteed (though it may happen). Scrolling is in
    /// discrete steps. It is up to the caller how to interpret such different step sizes.
    Wheel,
    /// Scrolling through tilting the scroll wheel.
    ///
    /// No terminating event is guaranteed (though it may happen). Scrolling is in
    /// discrete steps. It is up to the caller how to interpret such different step sizes.
    WheelTilt,
}

/// Direction of physical motion that caused pointer axis event
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AxisRelativeDirection {
    /// Physical motion matches axis direction
    Identical,
    /// Physical motion is inverse of axis direction (e.g. natural scrolling)
    Inverted,
}

#[cfg(feature = "wayland_frontend")]
impl From<AxisRelativeDirection> for wl_pointer::AxisRelativeDirection {
    #[inline]
    fn from(direction: AxisRelativeDirection) -> Self {
        match direction {
            AxisRelativeDirection::Identical => wl_pointer::AxisRelativeDirection::Identical,
            AxisRelativeDirection::Inverted => wl_pointer::AxisRelativeDirection::Inverted,
        }
    }
}

/// Trait for pointer events generated by scrolling on an axis.
pub trait PointerAxisEvent<B: InputBackend>: Event<B> {
    /// Amount of scrolling in pixels on the given [`Axis`].
    ///
    /// Guaranteed to be `Some` when source returns either [`AxisSource::Finger`] or [`AxisSource::Continuous`].
    fn amount(&self, axis: Axis) -> Option<f64>;

    /// Amount of scrolling in discrete steps on the given [`Axis`].
    ///
    /// Guaranteed to be `Some` when source returns either [`AxisSource::Wheel`] or [`AxisSource::WheelTilt`].
    fn amount_v120(&self, axis: Axis) -> Option<f64>;

    /// Source of the scroll event.
    fn source(&self) -> AxisSource;

    /// Relative direction of physical motion.
    fn relative_direction(&self, axis: Axis) -> AxisRelativeDirection;
}

impl<B: InputBackend> PointerAxisEvent<B> for UnusedEvent {
    fn amount(&self, _axis: Axis) -> Option<f64> {
        match *self {}
    }

    fn amount_v120(&self, _axis: Axis) -> Option<f64> {
        match *self {}
    }

    fn source(&self) -> AxisSource {
        match *self {}
    }

    fn relative_direction(&self, _axis: Axis) -> AxisRelativeDirection {
        match *self {}
    }
}

/// Trait for pointer events generated by relative device movement.
pub trait PointerMotionEvent<B: InputBackend>: Event<B> {
    /// Delta between the last and new pointer device position interpreted as pixel movement
    fn delta(&self) -> Point<f64, Logical> {
        (self.delta_x(), self.delta_y()).into()
    }

    /// Unaccelerated delta between the last and new pointer device position
    fn delta_unaccel(&self) -> Point<f64, Logical> {
        (self.delta_x_unaccel(), self.delta_y_unaccel()).into()
    }

    /// Delta on the x axis between the last and new pointer device position interpreted as pixel movement
    fn delta_x(&self) -> f64;

    /// Delta on the y axis between the last and new pointer device position interpreted as pixel movement
    fn delta_y(&self) -> f64;

    /// Unaccelerated delta on the x axis between the last and new pointer device position
    fn delta_x_unaccel(&self) -> f64;

    /// Unaccelerated delta on the y axis between the last and new pointer device position
    fn delta_y_unaccel(&self) -> f64;
}

impl<B: InputBackend> PointerMotionEvent<B> for UnusedEvent {
    fn delta_x(&self) -> f64 {
        match *self {}
    }

    fn delta_y(&self) -> f64 {
        match *self {}
    }

    fn delta_x_unaccel(&self) -> f64 {
        match *self {}
    }

    fn delta_y_unaccel(&self) -> f64 {
        match *self {}
    }
}

/// Trait for pointer events generated by absolute device positioning.
pub trait PointerMotionAbsoluteEvent<B: InputBackend>: AbsolutePositionEvent<B> {}
impl<B: InputBackend> PointerMotionAbsoluteEvent<B> for UnusedEvent {}

/// Input event with absolute location data.
pub trait AbsolutePositionEvent<B: InputBackend>: Event<B> {
    /// Device position in it's original coordinate space.
    ///
    /// The format is defined by the backend implementation.
    fn position(&self) -> Point<f64, Raw> {
        (self.x(), self.y()).into()
    }

    /// Device x position in it's original coordinate space.
    ///
    /// The format is defined by the backend implementation.
    fn x(&self) -> f64;

    /// Device y position in it's original coordinate space.
    ///
    /// The format is defined by the backend implementation.
    fn y(&self) -> f64;

    /// Device position converted to the targets coordinate space.
    /// E.g. the focused output's resolution.
    fn position_transformed(&self, coordinate_space: Size<i32, Logical>) -> Point<f64, Logical> {
        (
            self.x_transformed(coordinate_space.w),
            self.y_transformed(coordinate_space.h),
        )
            .into()
    }

    /// Device x position converted to the targets coordinate space's width.
    /// E.g. the focused output's width.
    fn x_transformed(&self, width: i32) -> f64;

    /// Device y position converted to the targets coordinate space's height.
    /// E.g. the focused output's height.
    fn y_transformed(&self, height: i32) -> f64;
}

impl<B: InputBackend> AbsolutePositionEvent<B> for UnusedEvent {
    fn x(&self) -> f64 {
        match *self {}
    }

    fn y(&self) -> f64 {
        match *self {}
    }

    fn x_transformed(&self, _width: i32) -> f64 {
        match *self {}
    }

    fn y_transformed(&self, _height: i32) -> f64 {
        match *self {}
    }
}

/// Slot of a different touch event.
///
/// Touch events are grouped by slots, usually to identify different
/// fingers on a multi-touch enabled input device. Events should only
/// be interpreted in the context of other events on the same slot.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TouchSlot {
    id: Option<u32>,
}

impl From<Option<u32>> for TouchSlot {
    #[inline]
    fn from(id: Option<u32>) -> Self {
        Self { id }
    }
}

impl From<TouchSlot> for i32 {
    #[inline]
    fn from(slot: TouchSlot) -> i32 {
        slot.id.map(|id| id as i32).unwrap_or(-1)
    }
}

/// Trait with functions available for all touch events.
pub trait TouchEvent<B: InputBackend>: Event<B> {
    /// Multi-touch slot identifier.
    fn slot(&self) -> TouchSlot;
}

impl<B: InputBackend> TouchEvent<B> for UnusedEvent {
    #[inline]
    fn slot(&self) -> TouchSlot {
        match *self {}
    }
}

/// Trait for touch events starting at a given position.
pub trait TouchDownEvent<B: InputBackend>: TouchEvent<B> + AbsolutePositionEvent<B> {}
impl<B: InputBackend> TouchDownEvent<B> for UnusedEvent {}

/// Trait for touch events regarding movement on the screen
pub trait TouchMotionEvent<B: InputBackend>: TouchEvent<B> + AbsolutePositionEvent<B> {}
impl<B: InputBackend> TouchMotionEvent<B> for UnusedEvent {}

/// Trait for touch events finishing.
pub trait TouchUpEvent<B: InputBackend>: TouchEvent<B> {}
impl<B: InputBackend> TouchUpEvent<B> for UnusedEvent {}

/// Trait for touch events canceling the chain
pub trait TouchCancelEvent<B: InputBackend>: TouchEvent<B> {}
impl<B: InputBackend> TouchCancelEvent<B> for UnusedEvent {}

/// Trait for touch frame events
pub trait TouchFrameEvent<B: InputBackend>: Event<B> {}
impl<B: InputBackend> TouchFrameEvent<B> for UnusedEvent {}

/// Types of Switches
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Switch {
    /// The laptop lid was closed when the [`SwitchState`] is
    /// [`On`](SwitchState::On), or was opened when it is [`Off`](SwitchState::Off)
    Lid,
    /// This switch indicates whether the device is in normal laptop mode
    /// or behaves like a tablet-like device where the primary
    /// interaction is usually a touch screen. When in tablet mode, the
    /// keyboard and touchpad are usually inaccessible.
    ///
    /// If the switch is in state [`SwitchState::Off`], the
    /// device is in laptop mode. If the switch is in state
    /// [`SwitchState::On`], the device is in tablet mode and the
    /// keyboard or touchpad may not be  accessible.
    ///
    /// It is up to the caller to identify which devices are inaccessible
    /// in tablet mode.
    TabletMode,
}

/// State of a Switch
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwitchState {
    /// Switch is off
    Off,
    /// Switch is on
    On,
}

/// Trait for switch toggle events
pub trait SwitchToggleEvent<B: InputBackend>: Event<B> {
    /// [`Switch`] which triggered the event
    fn switch(&self) -> Option<Switch>;
    /// [`State`](SwitchState) of the switch
    fn state(&self) -> SwitchState;
}
impl<B: InputBackend> SwitchToggleEvent<B> for UnusedEvent {
    fn switch(&self) -> Option<Switch> {
        match *self {}
    }
    fn state(&self) -> SwitchState {
        match *self {}
    }
}

/// Trait that describes objects providing a source of input events. All input backends
/// need to implement this and provide the same base guarantees about the precision of
/// given events.
pub trait InputBackend: Sized {
    /// Type representing input devices
    type Device: Device;
    /// Type representing keyboard events
    type KeyboardKeyEvent: KeyboardKeyEvent<Self>;
    /// Type representing axis events on pointer devices
    type PointerAxisEvent: PointerAxisEvent<Self>;
    /// Type representing button events on pointer devices
    type PointerButtonEvent: PointerButtonEvent<Self>;
    /// Type representing motion events of pointer devices
    type PointerMotionEvent: PointerMotionEvent<Self>;
    /// Type representing motion events of pointer devices
    type PointerMotionAbsoluteEvent: PointerMotionAbsoluteEvent<Self>;
    /// Type representing swipe begin events of pointer devices
    type GestureSwipeBeginEvent: GestureSwipeBeginEvent<Self>;
    /// Type representing swipe update events of pointer devices
    type GestureSwipeUpdateEvent: GestureSwipeUpdateEvent<Self>;
    /// Type representing swipe end events of pointer devices
    type GestureSwipeEndEvent: GestureSwipeEndEvent<Self>;
    /// Type representing pinch begin events of pointer devices
    type GesturePinchBeginEvent: GesturePinchBeginEvent<Self>;
    /// Type representing pinch update events of pointer devices
    type GesturePinchUpdateEvent: GesturePinchUpdateEvent<Self>;
    /// Type representing pinch end events of pointer devices
    type GesturePinchEndEvent: GesturePinchEndEvent<Self>;
    /// Type representing hold begin events of pointer devices
    type GestureHoldBeginEvent: GestureHoldBeginEvent<Self>;
    /// Type representing hold end events of pointer devices
    type GestureHoldEndEvent: GestureHoldEndEvent<Self>;
    /// Type representing touch events starting
    type TouchDownEvent: TouchDownEvent<Self>;
    /// Type representing touch events ending
    type TouchUpEvent: TouchUpEvent<Self>;
    /// Type representing touch events from moving
    type TouchMotionEvent: TouchMotionEvent<Self>;
    /// Type representing canceling of touch events
    type TouchCancelEvent: TouchCancelEvent<Self>;
    /// Type representing touch frame events
    type TouchFrameEvent: TouchFrameEvent<Self>;
    /// Type representing axis events on tablet devices
    type TabletToolAxisEvent: TabletToolAxisEvent<Self>;
    /// Type representing proximity events on tablet devices
    type TabletToolProximityEvent: TabletToolProximityEvent<Self>;
    /// Type representing tip events on tablet devices
    type TabletToolTipEvent: TabletToolTipEvent<Self>;
    /// Type representing button events on tablet tool devices
    type TabletToolButtonEvent: TabletToolButtonEvent<Self>;
    /// Type representing switch toggle events
    type SwitchToggleEvent: SwitchToggleEvent<Self>;

    /// Special events that are custom to this backend
    type SpecialEvent;
}

/// Different events that can be generated by an input backend
#[derive(Debug)]
pub enum InputEvent<B: InputBackend> {
    /// An input device was connected
    DeviceAdded {
        /// The added device
        device: B::Device,
    },
    /// An input device was disconnected
    DeviceRemoved {
        /// The removed device
        device: B::Device,
    },
    /// A keyboard event occurred
    Keyboard {
        /// The keyboard event
        event: B::KeyboardKeyEvent,
    },
    /// A relative pointer motion occurred
    PointerMotion {
        /// The pointer motion event
        event: B::PointerMotionEvent,
    },
    /// An absolute pointer motion occurs
    PointerMotionAbsolute {
        /// The absolute pointer motion event
        event: B::PointerMotionAbsoluteEvent,
    },
    /// A pointer button was pressed or released
    PointerButton {
        /// The pointer button event
        event: B::PointerButtonEvent,
    },
    /// A pointer action occurred while scrolling on an axis
    PointerAxis {
        /// The pointer axis event
        event: B::PointerAxisEvent,
    },
    /// A pointer swipe gesture began
    GestureSwipeBegin {
        /// The gesture event
        event: B::GestureSwipeBeginEvent,
    },
    /// A pointer swipe gesture updated
    GestureSwipeUpdate {
        /// The gesture event
        event: B::GestureSwipeUpdateEvent,
    },
    /// A pointer swipe gesture ended
    GestureSwipeEnd {
        /// The gesture event
        event: B::GestureSwipeEndEvent,
    },
    /// A pointer pinch gesture began
    GesturePinchBegin {
        /// The gesture event
        event: B::GesturePinchBeginEvent,
    },
    /// A pointer pinch gesture updated
    GesturePinchUpdate {
        /// The gesture event
        event: B::GesturePinchUpdateEvent,
    },
    /// A pointer pinch gesture ended
    GesturePinchEnd {
        /// The gesture event
        event: B::GesturePinchEndEvent,
    },
    /// A pointer hold gesture began
    GestureHoldBegin {
        /// The gesture event
        event: B::GestureHoldBeginEvent,
    },
    /// A pointer hold gesture ended
    GestureHoldEnd {
        /// The gesture event
        event: B::GestureHoldEndEvent,
    },
    /// A new touchpoint appeared
    TouchDown {
        /// The touch down event
        event: B::TouchDownEvent,
    },
    /// A touchpoint moved
    TouchMotion {
        /// The touch motion event
        event: B::TouchMotionEvent,
    },
    /// A touchpoint was removed
    TouchUp {
        /// The touch up event
        event: B::TouchUpEvent,
    },
    /// A touch sequence was cancelled
    TouchCancel {
        /// The touch cancel event
        event: B::TouchCancelEvent,
    },
    /// A touch frame was emitted
    ///
    /// A set of two events received on the same seat between two frames should
    /// be interpreted as an atomic event.
    TouchFrame {
        /// The touch frame event
        event: B::TouchFrameEvent,
    },

    /// A tablet tool axis was emitted
    TabletToolAxis {
        /// The tablet tool axis event
        event: B::TabletToolAxisEvent,
    },

    /// A tablet tool proximity was emitted
    TabletToolProximity {
        /// The tablet tool proximity  event
        event: B::TabletToolProximityEvent,
    },

    /// A tablet tool tip event was emitted
    TabletToolTip {
        /// The tablet tool axis event
        event: B::TabletToolTipEvent,
    },

    /// A tablet tool button was pressed or released
    TabletToolButton {
        /// The pointer button event
        event: B::TabletToolButtonEvent,
    },

    /// A switch was toggled
    SwitchToggle {
        /// The switch toggle event
        event: B::SwitchToggleEvent,
    },

    /// Special event specific of this backend
    Special(B::SpecialEvent),
}

/// Converts an xorg mouse button to the format used by libinput.
///
/// Taken from https://sources.debian.org/src/xserver-xorg-input-libinput/1.1.0-1/src/xf86libinput.c/?hl=1508#L236-L252
#[cfg(any(feature = "backend_winit", feature = "backend_x11"))]
pub(crate) fn xorg_mouse_to_libinput(xorg: u32) -> u32 {
    match xorg {
        0 => 0,
        1 => 0x110,            // BTN_LEFT
        2 => 0x112,            // BTN_MIDDLE
        3 => 0x111,            // BTN_RIGHT
        _ => xorg - 8 + 0x113, // BTN_SIZE
    }
}
