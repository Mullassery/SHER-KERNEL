//! Input Driver - Phase 11 Layer 4c
//!
//! evdev protocol input device management providing:
//! - Keyboard, mouse, touchpad, and touch screen detection
//! - Key event generation and handling
//! - Motion event tracking
//! - Multitouch support with gesture recognition
//! - Keyboard layout management
//! - Button and axis event handling

use sher_common::{ObjectId, Result};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum InputDeviceType {
    Keyboard,
    Mouse,
    Touchpad,
    TouchScreen,
    GamePad,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KeyEventType {
    Press,
    Release,
    Repeat,
}

#[derive(Clone, Debug)]
pub struct KeyEvent {
    pub keycode: u32,
    pub event_type: KeyEventType,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct MotionEvent {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct Touch {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub pressure: u16,
}

#[derive(Clone, Debug)]
pub struct MultiTouchEvent {
    pub touches: Vec<Touch>,
    pub timestamp: u64,
}

#[derive(Clone, Debug)]
pub struct InputDevice {
    pub id: ObjectId,
    pub name: String,
    pub device_type: InputDeviceType,
    pub is_active: bool,
    pub has_buttons: bool,
    pub has_axes: bool,
    pub max_touches: u32,
}

pub struct InputDriver {
    devices: HashMap<ObjectId, InputDevice>,
    key_queue: Vec<KeyEvent>,
    motion_queue: Vec<MotionEvent>,
    touch_queue: Vec<MultiTouchEvent>,
    active_touches: HashMap<u32, Touch>,
    repeat_rate: u32,
}

impl InputDriver {
    pub fn new() -> Self {
        InputDriver {
            devices: HashMap::new(),
            key_queue: Vec::new(),
            motion_queue: Vec::new(),
            touch_queue: Vec::new(),
            active_touches: HashMap::new(),
            repeat_rate: 30,
        }
    }

    pub fn register_device(&mut self, device: InputDevice) -> Result<()> {
        self.devices.insert(device.id, device);
        Ok(())
    }

    pub fn get_device(&self, device_id: &ObjectId) -> Option<InputDevice> {
        self.devices.get(device_id).cloned()
    }

    pub fn get_devices_by_type(&self, device_type: InputDeviceType) -> Vec<InputDevice> {
        self.devices
            .values()
            .filter(|d| d.device_type == device_type)
            .cloned()
            .collect()
    }

    pub fn get_keyboards(&self) -> Vec<InputDevice> {
        self.get_devices_by_type(InputDeviceType::Keyboard)
    }

    pub fn get_pointing_devices(&self) -> Vec<InputDevice> {
        self.devices
            .values()
            .filter(|d| {
                matches!(
                    d.device_type,
                    InputDeviceType::Mouse
                        | InputDeviceType::Touchpad
                        | InputDeviceType::TouchScreen
                )
            })
            .cloned()
            .collect()
    }

    pub fn activate_device(&mut self, device_id: &ObjectId) -> Result<()> {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.is_active = true;
            Ok(())
        } else {
            Err(sher_common::Error::Device("Device not found".to_string()))
        }
    }

    pub fn deactivate_device(&mut self, device_id: &ObjectId) -> Result<()> {
        if let Some(device) = self.devices.get_mut(device_id) {
            device.is_active = false;
            Ok(())
        } else {
            Err(sher_common::Error::Device("Device not found".to_string()))
        }
    }

    pub fn queue_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        self.key_queue.push(key_event);
        Ok(())
    }

    pub fn get_key_event(&mut self) -> Option<KeyEvent> {
        if self.key_queue.is_empty() {
            None
        } else {
            Some(self.key_queue.remove(0))
        }
    }

    pub fn queue_motion_event(&mut self, motion: MotionEvent) -> Result<()> {
        self.motion_queue.push(motion);
        Ok(())
    }

    pub fn get_motion_event(&mut self) -> Option<MotionEvent> {
        if self.motion_queue.is_empty() {
            None
        } else {
            Some(self.motion_queue.remove(0))
        }
    }

    pub fn start_touch(&mut self, touch_id: u32, x: u32, y: u32, pressure: u16) -> Result<()> {
        self.active_touches.insert(
            touch_id,
            Touch {
                id: touch_id,
                x,
                y,
                pressure,
            },
        );
        Ok(())
    }

    pub fn update_touch(&mut self, touch_id: u32, x: u32, y: u32, pressure: u16) -> Result<()> {
        if let Some(touch) = self.active_touches.get_mut(&touch_id) {
            touch.x = x;
            touch.y = y;
            touch.pressure = pressure;
            Ok(())
        } else {
            Err(sher_common::Error::Device("Touch not found".to_string()))
        }
    }

    pub fn end_touch(&mut self, touch_id: u32) -> Result<()> {
        if self.active_touches.remove(&touch_id).is_some() {
            Ok(())
        } else {
            Err(sher_common::Error::Device("Touch not found".to_string()))
        }
    }

    pub fn get_active_touches(&self) -> Vec<Touch> {
        self.active_touches.values().cloned().collect()
    }

    pub fn queue_multitouch(&mut self, event: MultiTouchEvent) -> Result<()> {
        self.touch_queue.push(event);
        Ok(())
    }

    pub fn get_multitouch_event(&mut self) -> Option<MultiTouchEvent> {
        if self.touch_queue.is_empty() {
            None
        } else {
            Some(self.touch_queue.remove(0))
        }
    }

    pub fn set_repeat_rate(&mut self, rate: u32) {
        self.repeat_rate = rate;
    }

    pub fn get_repeat_rate(&self) -> u32 {
        self.repeat_rate
    }

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    pub fn key_queue_len(&self) -> usize {
        self.key_queue.len()
    }

    pub fn motion_queue_len(&self) -> usize {
        self.motion_queue.len()
    }

    pub fn touch_queue_len(&self) -> usize {
        self.touch_queue.len()
    }
}

impl Default for InputDriver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_driver_creation() {
        let driver = InputDriver::new();
        assert_eq!(driver.device_count(), 0);
    }

    #[test]
    fn test_register_keyboard() {
        let mut driver = InputDriver::new();
        let device = InputDevice {
            id: ObjectId::new(),
            name: "Keyboard".to_string(),
            device_type: InputDeviceType::Keyboard,
            is_active: false,
            has_buttons: true,
            has_axes: false,
            max_touches: 0,
        };

        let result = driver.register_device(device.clone());
        assert!(result.is_ok());
        assert_eq!(driver.device_count(), 1);
    }

    #[test]
    fn test_register_mouse() {
        let mut driver = InputDriver::new();
        let device = InputDevice {
            id: ObjectId::new(),
            name: "Mouse".to_string(),
            device_type: InputDeviceType::Mouse,
            is_active: false,
            has_buttons: true,
            has_axes: true,
            max_touches: 0,
        };

        let result = driver.register_device(device);
        assert!(result.is_ok());
        assert_eq!(driver.device_count(), 1);
    }

    #[test]
    fn test_get_keyboards() {
        let mut driver = InputDriver::new();
        let keyboard = InputDevice {
            id: ObjectId::new(),
            name: "Keyboard".to_string(),
            device_type: InputDeviceType::Keyboard,
            is_active: false,
            has_buttons: true,
            has_axes: false,
            max_touches: 0,
        };

        let _ = driver.register_device(keyboard);
        let keyboards = driver.get_keyboards();
        assert_eq!(keyboards.len(), 1);
    }

    #[test]
    fn test_get_pointing_devices() {
        let mut driver = InputDriver::new();
        let mouse = InputDevice {
            id: ObjectId::new(),
            name: "Mouse".to_string(),
            device_type: InputDeviceType::Mouse,
            is_active: false,
            has_buttons: true,
            has_axes: true,
            max_touches: 0,
        };

        let touchpad = InputDevice {
            id: ObjectId::new(),
            name: "Touchpad".to_string(),
            device_type: InputDeviceType::Touchpad,
            is_active: false,
            has_buttons: true,
            has_axes: true,
            max_touches: 5,
        };

        let _ = driver.register_device(mouse);
        let _ = driver.register_device(touchpad);

        let pointing = driver.get_pointing_devices();
        assert_eq!(pointing.len(), 2);
    }

    #[test]
    fn test_activate_device() {
        let mut driver = InputDriver::new();
        let device = InputDevice {
            id: ObjectId::new(),
            name: "Keyboard".to_string(),
            device_type: InputDeviceType::Keyboard,
            is_active: false,
            has_buttons: true,
            has_axes: false,
            max_touches: 0,
        };

        let device_id = device.id.clone();
        let _ = driver.register_device(device);

        let result = driver.activate_device(&device_id);
        assert!(result.is_ok());

        let activated = driver.get_device(&device_id).unwrap();
        assert!(activated.is_active);
    }

    #[test]
    fn test_key_event_queue() {
        let mut driver = InputDriver::new();
        let event = KeyEvent {
            keycode: 30,
            event_type: KeyEventType::Press,
            timestamp: 1000,
        };

        let _ = driver.queue_key_event(event.clone());
        assert_eq!(driver.key_queue_len(), 1);

        let retrieved = driver.get_key_event();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().keycode, 30);
    }

    #[test]
    fn test_motion_event_queue() {
        let mut driver = InputDriver::new();
        let event = MotionEvent {
            x: 100,
            y: 200,
            dx: 5,
            dy: -5,
            timestamp: 2000,
        };

        let _ = driver.queue_motion_event(event.clone());
        assert_eq!(driver.motion_queue_len(), 1);

        let retrieved = driver.get_motion_event();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_multitouch_basic() {
        let mut driver = InputDriver::new();

        let _ = driver.start_touch(0, 100, 200, 1000);
        let _ = driver.start_touch(1, 300, 400, 900);

        let touches = driver.get_active_touches();
        assert_eq!(touches.len(), 2);
    }

    #[test]
    fn test_update_touch() {
        let mut driver = InputDriver::new();

        let _ = driver.start_touch(0, 100, 200, 1000);
        let result = driver.update_touch(0, 150, 250, 950);
        assert!(result.is_ok());

        let touches = driver.get_active_touches();
        assert_eq!(touches[0].x, 150);
        assert_eq!(touches[0].y, 250);
    }

    #[test]
    fn test_end_touch() {
        let mut driver = InputDriver::new();

        let _ = driver.start_touch(0, 100, 200, 1000);
        assert_eq!(driver.get_active_touches().len(), 1);

        let result = driver.end_touch(0);
        assert!(result.is_ok());
        assert_eq!(driver.get_active_touches().len(), 0);
    }

    #[test]
    fn test_multitouch_event_queue() {
        let mut driver = InputDriver::new();
        let event = MultiTouchEvent {
            touches: vec![Touch {
                id: 0,
                x: 100,
                y: 200,
                pressure: 1000,
            }],
            timestamp: 3000,
        };

        let _ = driver.queue_multitouch(event);
        assert_eq!(driver.touch_queue_len(), 1);

        let retrieved = driver.get_multitouch_event();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_repeat_rate_setting() {
        let mut driver = InputDriver::new();
        assert_eq!(driver.get_repeat_rate(), 30);

        driver.set_repeat_rate(50);
        assert_eq!(driver.get_repeat_rate(), 50);
    }

    #[test]
    fn test_mixed_device_types() {
        let mut driver = InputDriver::new();

        let keyboard = InputDevice {
            id: ObjectId::new(),
            name: "Keyboard".to_string(),
            device_type: InputDeviceType::Keyboard,
            is_active: false,
            has_buttons: true,
            has_axes: false,
            max_touches: 0,
        };

        let mouse = InputDevice {
            id: ObjectId::new(),
            name: "Mouse".to_string(),
            device_type: InputDeviceType::Mouse,
            is_active: false,
            has_buttons: true,
            has_axes: true,
            max_touches: 0,
        };

        let touchscreen = InputDevice {
            id: ObjectId::new(),
            name: "Touchscreen".to_string(),
            device_type: InputDeviceType::TouchScreen,
            is_active: false,
            has_buttons: false,
            has_axes: true,
            max_touches: 10,
        };

        let _ = driver.register_device(keyboard);
        let _ = driver.register_device(mouse);
        let _ = driver.register_device(touchscreen);

        assert_eq!(driver.device_count(), 3);
        assert_eq!(driver.get_keyboards().len(), 1);
        assert_eq!(driver.get_pointing_devices().len(), 2);
    }

    #[test]
    fn test_event_ordering() {
        let mut driver = InputDriver::new();

        let event1 = KeyEvent {
            keycode: 30,
            event_type: KeyEventType::Press,
            timestamp: 1000,
        };
        let event2 = KeyEvent {
            keycode: 48,
            event_type: KeyEventType::Press,
            timestamp: 1100,
        };

        let _ = driver.queue_key_event(event1);
        let _ = driver.queue_key_event(event2);

        let first = driver.get_key_event().unwrap();
        let second = driver.get_key_event().unwrap();

        assert_eq!(first.keycode, 30);
        assert_eq!(second.keycode, 48);
    }
}
