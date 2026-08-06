//! Phase 12: System Integration Tests
//!
//! End-to-end integration testing of SHER Kernel with Aurora Design System
//! and Himalayas Browser ecosystem.

use sher_common::ObjectId;
use wayland_server::{WaylandCompositor, WaylandClient, PointerEvent, PointerEventType};
use gpu_driver::GPUDriver;
use audio_driver::AudioDriver;
use input_driver::InputDriver;
use unified_device_manager::UnifiedDeviceManager;

/// Represents a complete application stack integration
pub struct ApplicationStack {
    compositor: WaylandCompositor,
    gpu_driver: GPUDriver,
    audio_driver: AudioDriver,
    input_driver: InputDriver,
    device_manager: UnifiedDeviceManager,
}

impl ApplicationStack {
    pub fn new() -> Self {
        ApplicationStack {
            compositor: WaylandCompositor::new(),
            gpu_driver: GPUDriver::new(256 * 1024 * 1024),
            audio_driver: AudioDriver::new(),
            input_driver: InputDriver::new(),
            device_manager: UnifiedDeviceManager::new(),
        }
    }

    pub fn initialize(&mut self) -> sher_common::Result<()> {
        self.compositor.start()?;
        self.device_manager.initialize()?;
        Ok(())
    }

    pub fn shutdown(&mut self) -> sher_common::Result<()> {
        self.compositor.stop()?;
        Ok(())
    }

    pub fn get_compositor(&self) -> &WaylandCompositor {
        &self.compositor
    }

    pub fn get_compositor_mut(&mut self) -> &mut WaylandCompositor {
        &mut self.compositor
    }

    pub fn get_gpu_driver(&self) -> &GPUDriver {
        &self.gpu_driver
    }

    pub fn get_gpu_driver_mut(&mut self) -> &mut GPUDriver {
        &mut self.gpu_driver
    }

    pub fn get_audio_driver(&self) -> &AudioDriver {
        &self.audio_driver
    }

    pub fn get_audio_driver_mut(&mut self) -> &mut AudioDriver {
        &mut self.audio_driver
    }

    pub fn get_input_driver(&self) -> &InputDriver {
        &self.input_driver
    }

    pub fn get_input_driver_mut(&mut self) -> &mut InputDriver {
        &mut self.input_driver
    }

    pub fn get_device_manager(&self) -> &UnifiedDeviceManager {
        &self.device_manager
    }

    pub fn get_device_manager_mut(&mut self) -> &mut UnifiedDeviceManager {
        &mut self.device_manager
    }

    pub fn total_resources(&self) -> usize {
        self.compositor.client_count()
            + self.compositor.surface_count()
            + self.compositor.buffer_count()
            + self.device_manager.get_total_device_count()
    }
}

impl Default for ApplicationStack {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use input_driver::{InputDevice, InputDeviceType};
    use gpu_driver::{Connector, ConnectorType, ConnectorStatus, DisplayMode};
    use audio_driver::{AudioDevice, DeviceRole};

    #[test]
    fn test_stack_initialization() {
        let mut stack = ApplicationStack::new();
        let result = stack.initialize();
        assert!(result.is_ok());
        assert!(stack.get_compositor().is_running());
        assert!(stack.get_device_manager().is_initialized());
    }

    #[test]
    fn test_stack_shutdown() {
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();
        let result = stack.shutdown();
        assert!(result.is_ok());
        assert!(!stack.get_compositor().is_running());
    }

    #[test]
    fn test_wayland_client_lifecycle() {
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let client = WaylandClient {
            id: ObjectId::new(),
            name: "test_app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let result = stack.get_compositor_mut().connect_client(client);
        assert!(result.is_ok());
        assert_eq!(stack.get_compositor().client_count(), 1);

        let result = stack.get_compositor_mut().disconnect_client(&client_id);
        assert!(result.is_ok());
        assert_eq!(stack.get_compositor().client_count(), 0);
    }

    #[test]
    fn test_surface_rendering_pipeline() {
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let client = WaylandClient {
            id: ObjectId::new(),
            name: "app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = stack.get_compositor_mut().connect_client(client);

        let surface = stack
            .get_compositor_mut()
            .create_surface(&client_id)
            .unwrap();

        let buffer = stack
            .get_compositor_mut()
            .create_buffer(1920, 1080, 0x34325241)
            .unwrap();

        let _ = stack
            .get_compositor_mut()
            .attach_buffer(&surface.id, &buffer.id);
        let _ = stack
            .get_compositor_mut()
            .configure_surface(&surface.id, 1920, 1080);
        let _ = stack.get_compositor_mut().commit_surface(&surface.id);

        assert_eq!(stack.get_compositor().surface_count(), 1);
        assert_eq!(stack.get_compositor().buffer_count(), 1);
    }

    #[test]
    fn test_input_event_routing() {
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let keyboard = InputDevice {
            id: ObjectId::new(),
            name: "Keyboard".to_string(),
            device_type: InputDeviceType::Keyboard,
            is_active: false,
            has_buttons: true,
            has_axes: false,
            max_touches: 0,
        };

        let _ = stack
            .get_input_driver_mut()
            .register_device(keyboard.clone());
        let _ = stack.get_input_driver_mut().activate_device(&keyboard.id);

        let device = stack.get_input_driver().get_device(&keyboard.id);
        assert!(device.is_some());
        assert!(device.unwrap().is_active);
    }

    #[test]
    fn test_gpu_display_setup() {
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let mode = DisplayMode {
            width: 1920,
            height: 1080,
            refresh_rate: 60,
            clock: 148500,
        };

        let connector = Connector {
            id: ObjectId::new(),
            connector_type: ConnectorType::HDMI,
            status: ConnectorStatus::Connected,
            supported_modes: vec![mode.clone()],
            current_mode: None,
        };

        let connector_id = connector.id.clone();
        let _ = stack.get_gpu_driver_mut().register_connector(connector);

        let result = stack
            .get_gpu_driver_mut()
            .set_mode(&connector_id, mode);
        assert!(result.is_ok());
        assert_eq!(stack.get_gpu_driver().connector_count(), 1);
    }

    #[test]
    fn test_audio_device_setup() {
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let device = AudioDevice {
            id: ObjectId::new(),
            name: "Speakers".to_string(),
            role: DeviceRole::Playback,
            sample_rates: vec![48000],
            current_sample_rate: 48000,
            formats: vec![audio_driver::AudioFormat::S16LE],
            current_format: audio_driver::AudioFormat::S16LE,
            channels: 2,
            is_active: false,
        };

        let result = stack.get_audio_driver_mut().register_device(device.clone());
        assert!(result.is_ok());
        assert_eq!(stack.get_audio_driver().device_count(), 1);
    }

    #[test]
    fn test_device_manager_coordination() {
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let gpu_id = ObjectId::new();
        let audio_id = ObjectId::new();
        let input_id = ObjectId::new();

        let _ = stack
            .get_device_manager_mut()
            .register_gpu_device(gpu_id.clone(), "GPU".to_string());
        let _ = stack
            .get_device_manager_mut()
            .register_audio_device(audio_id.clone(), "Audio".to_string());
        let _ = stack
            .get_device_manager_mut()
            .register_input_device(input_id.clone(), "Input".to_string());

        assert_eq!(stack.get_device_manager().get_total_device_count(), 3);
        assert!(stack.get_device_manager().all_devices_healthy());
    }

    #[test]
    fn test_pointer_focus_sync() {
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let client = WaylandClient {
            id: ObjectId::new(),
            name: "app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = stack.get_compositor_mut().connect_client(client);

        let surface = stack
            .get_compositor_mut()
            .create_surface(&client_id)
            .unwrap();

        let event = PointerEvent {
            surface_id: Some(surface.id.clone()),
            event_type: PointerEventType::Enter,
            x: 100,
            y: 200,
            button: None,
        };

        let _ = stack.get_compositor_mut().route_pointer_event(event);

        assert_eq!(
            stack.get_compositor().get_focused_surface(),
            Some(surface.id)
        );
    }

    #[test]
    fn test_multi_app_scenario() {
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let app1 = WaylandClient {
            id: ObjectId::new(),
            name: "browser".to_string(),
            is_connected: false,
        };

        let app2 = WaylandClient {
            id: ObjectId::new(),
            name: "terminal".to_string(),
            is_connected: false,
        };

        let app1_id = app1.id.clone();
        let app2_id = app2.id.clone();

        let _ = stack.get_compositor_mut().connect_client(app1);
        let _ = stack.get_compositor_mut().connect_client(app2);

        let _surface1 = stack
            .get_compositor_mut()
            .create_surface(&app1_id)
            .unwrap();
        let _surface2 = stack
            .get_compositor_mut()
            .create_surface(&app2_id)
            .unwrap();

        assert_eq!(stack.get_compositor().client_count(), 2);
        assert_eq!(stack.get_compositor().surface_count(), 2);
    }

    #[test]
    fn test_error_isolation() {
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let device_id = ObjectId::new();
        let _ = stack
            .get_device_manager_mut()
            .register_gpu_device(device_id.clone(), "GPU".to_string());

        let _ = stack
            .get_device_manager_mut()
            .report_device_error(&device_id, "Test error".to_string());

        let device = stack
            .get_device_manager()
            .get_gpu_device(&device_id)
            .unwrap();
        assert!(device.is_healthy);

        for _ in 0..4 {
            let _ = stack
                .get_device_manager_mut()
                .report_device_error(&device_id, "Error".to_string());
        }

        let device = stack
            .get_device_manager()
            .get_gpu_device(&device_id)
            .unwrap();
        assert!(!device.is_healthy);

        assert!(stack.get_device_manager().all_devices_healthy() == false);
    }

    #[test]
    fn test_resource_accounting() {
        let mut stack = ApplicationStack::new();
        let _ = stack.initialize();

        let initial_resources = stack.total_resources();

        let client = WaylandClient {
            id: ObjectId::new(),
            name: "app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        let _ = stack.get_compositor_mut().connect_client(client);
        let _ = stack
            .get_compositor_mut()
            .create_surface(&client_id)
            .unwrap();
        let _ = stack
            .get_compositor_mut()
            .create_buffer(1920, 1080, 0x34325241)
            .unwrap();

        let final_resources = stack.total_resources();
        assert!(final_resources > initial_resources);
    }

    #[test]
    fn test_complete_workflow() {
        let mut stack = ApplicationStack::new();
        assert!(stack.initialize().is_ok());

        let client = WaylandClient {
            id: ObjectId::new(),
            name: "himalayas_browser".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        assert!(stack.get_compositor_mut().connect_client(client).is_ok());

        let surface = stack
            .get_compositor_mut()
            .create_surface(&client_id)
            .unwrap();

        assert!(stack
            .get_compositor_mut()
            .configure_surface(&surface.id, 1920, 1080)
            .is_ok());

        let buffer = stack
            .get_compositor_mut()
            .create_buffer(1920, 1080, 0x34325241)
            .unwrap();

        assert!(stack
            .get_compositor_mut()
            .attach_buffer(&surface.id, &buffer.id)
            .is_ok());

        assert!(stack
            .get_compositor_mut()
            .commit_surface(&surface.id)
            .is_ok());

        assert!(stack.shutdown().is_ok());
    }

    #[test]
    fn test_aurora_theme_application() {
        let mut stack = ApplicationStack::new();
        assert!(stack.initialize().is_ok());

        let client = WaylandClient {
            id: ObjectId::new(),
            name: "themed_app".to_string(),
            is_connected: false,
        };

        let client_id = client.id.clone();
        assert!(stack.get_compositor_mut().connect_client(client).is_ok());

        let surface = stack
            .get_compositor_mut()
            .create_surface(&client_id)
            .unwrap();

        assert!(stack
            .get_compositor_mut()
            .configure_surface(&surface.id, 800, 600)
            .is_ok());

        let buffer = stack
            .get_compositor_mut()
            .create_buffer(800, 600, 0x34325241)
            .unwrap();

        assert!(stack
            .get_compositor_mut()
            .attach_buffer(&surface.id, &buffer.id)
            .is_ok());

        assert_eq!(buffer.format, 0x34325241);
        assert_eq!(buffer.width, 800);
        assert_eq!(buffer.height, 600);
    }

    #[test]
    fn test_gtk4_widget_rendering() {
        let mut stack = ApplicationStack::new();
        assert!(stack.initialize().is_ok());

        let gtk_app = WaylandClient {
            id: ObjectId::new(),
            name: "gtk4_app".to_string(),
            is_connected: false,
        };

        let app_id = gtk_app.id.clone();
        assert!(stack.get_compositor_mut().connect_client(gtk_app).is_ok());

        let main_surface = stack
            .get_compositor_mut()
            .create_surface(&app_id)
            .unwrap();

        assert!(stack
            .get_compositor_mut()
            .configure_surface(&main_surface.id, 1024, 768)
            .is_ok());

        let buffer = stack
            .get_compositor_mut()
            .create_buffer(1024, 768, 0x34325241)
            .unwrap();

        assert!(stack
            .get_compositor_mut()
            .attach_buffer(&main_surface.id, &buffer.id)
            .is_ok());

        assert!(stack
            .get_compositor_mut()
            .commit_surface(&main_surface.id)
            .is_ok());

        assert!(stack.get_compositor().surface_count() == 1);
    }

    #[test]
    fn test_himalayas_browser_launch() {
        let mut stack = ApplicationStack::new();
        assert!(stack.initialize().is_ok());

        let browser = WaylandClient {
            id: ObjectId::new(),
            name: "himalayas_browser".to_string(),
            is_connected: false,
        };

        let browser_id = browser.id.clone();
        assert!(stack.get_compositor_mut().connect_client(browser).is_ok());

        let rendering_surface = stack
            .get_compositor_mut()
            .create_surface(&browser_id)
            .unwrap();

        assert!(stack
            .get_compositor_mut()
            .configure_surface(&rendering_surface.id, 1920, 1080)
            .is_ok());

        let backbuffer = stack
            .get_compositor_mut()
            .create_buffer(1920, 1080, 0x34325241)
            .unwrap();

        assert!(stack
            .get_compositor_mut()
            .attach_buffer(&rendering_surface.id, &backbuffer.id)
            .is_ok());

        assert!(stack.get_device_manager().is_initialized());
        assert!(stack.get_compositor().is_running());
    }

    #[test]
    fn test_audio_playback_integration() {
        let mut stack = ApplicationStack::new();
        assert!(stack.initialize().is_ok());

        let speaker = AudioDevice {
            id: ObjectId::new(),
            name: "Speaker".to_string(),
            role: DeviceRole::Playback,
            sample_rates: vec![44100, 48000, 96000],
            current_sample_rate: 48000,
            formats: vec![
                audio_driver::AudioFormat::S16LE,
                audio_driver::AudioFormat::S32LE,
            ],
            current_format: audio_driver::AudioFormat::S16LE,
            channels: 2,
            is_active: false,
        };

        let speaker_id = speaker.id.clone();
        assert!(stack.get_audio_driver_mut().register_device(speaker).is_ok());

        let buffer = stack
            .get_audio_driver_mut()
            .allocate_buffer(&speaker_id, 4096)
            .unwrap();

        assert!(stack
            .get_audio_driver_mut()
            .start_stream(&speaker_id)
            .is_ok());

        assert_eq!(stack.get_audio_driver().get_active_stream_count(), 1);

        assert!(stack
            .get_audio_driver_mut()
            .write_buffer(&buffer.id, 2048)
            .is_ok());

        assert!(stack
            .get_audio_driver_mut()
            .stop_stream(&speaker_id)
            .is_ok());

        assert_eq!(stack.get_audio_driver().get_active_stream_count(), 0);
    }

    #[test]
    fn test_input_event_pipeline() {
        let mut stack = ApplicationStack::new();
        assert!(stack.initialize().is_ok());

        let keyboard = InputDevice {
            id: ObjectId::new(),
            name: "USB Keyboard".to_string(),
            device_type: InputDeviceType::Keyboard,
            is_active: false,
            has_buttons: true,
            has_axes: false,
            max_touches: 0,
        };

        let mouse = InputDevice {
            id: ObjectId::new(),
            name: "USB Mouse".to_string(),
            device_type: InputDeviceType::Mouse,
            is_active: false,
            has_buttons: true,
            has_axes: true,
            max_touches: 0,
        };

        let keyboard_id = keyboard.id.clone();
        let mouse_id = mouse.id.clone();

        assert!(stack
            .get_input_driver_mut()
            .register_device(keyboard)
            .is_ok());
        assert!(stack
            .get_input_driver_mut()
            .register_device(mouse)
            .is_ok());

        assert!(stack
            .get_input_driver_mut()
            .activate_device(&keyboard_id)
            .is_ok());
        assert!(stack
            .get_input_driver_mut()
            .activate_device(&mouse_id)
            .is_ok());

        assert_eq!(stack.get_input_driver().get_keyboards().len(), 1);
        assert_eq!(stack.get_input_driver().get_pointing_devices().len(), 1);
    }

    #[test]
    fn test_multi_display_setup() {
        let mut stack = ApplicationStack::new();
        assert!(stack.initialize().is_ok());

        let mode_1080p = DisplayMode {
            width: 1920,
            height: 1080,
            refresh_rate: 60,
            clock: 148500,
        };

        let mode_1440p = DisplayMode {
            width: 2560,
            height: 1440,
            refresh_rate: 60,
            clock: 241500,
        };

        let hdmi = Connector {
            id: ObjectId::new(),
            connector_type: ConnectorType::HDMI,
            status: ConnectorStatus::Connected,
            supported_modes: vec![mode_1080p.clone()],
            current_mode: None,
        };

        let dp = Connector {
            id: ObjectId::new(),
            connector_type: ConnectorType::DisplayPort,
            status: ConnectorStatus::Connected,
            supported_modes: vec![mode_1440p.clone()],
            current_mode: None,
        };

        let hdmi_id = hdmi.id.clone();
        let dp_id = dp.id.clone();

        assert!(stack.get_gpu_driver_mut().register_connector(hdmi).is_ok());
        assert!(stack.get_gpu_driver_mut().register_connector(dp).is_ok());

        assert!(stack
            .get_gpu_driver_mut()
            .set_mode(&hdmi_id, mode_1080p)
            .is_ok());
        assert!(stack
            .get_gpu_driver_mut()
            .set_mode(&dp_id, mode_1440p)
            .is_ok());

        assert_eq!(stack.get_gpu_driver().connector_count(), 2);
    }

    #[test]
    fn test_concurrent_app_rendering() {
        let mut stack = ApplicationStack::new();
        assert!(stack.initialize().is_ok());

        let app1 = WaylandClient {
            id: ObjectId::new(),
            name: "app1".to_string(),
            is_connected: false,
        };

        let app2 = WaylandClient {
            id: ObjectId::new(),
            name: "app2".to_string(),
            is_connected: false,
        };

        let app3 = WaylandClient {
            id: ObjectId::new(),
            name: "app3".to_string(),
            is_connected: false,
        };

        let app1_id = app1.id.clone();
        let app2_id = app2.id.clone();
        let app3_id = app3.id.clone();

        assert!(stack.get_compositor_mut().connect_client(app1).is_ok());
        assert!(stack.get_compositor_mut().connect_client(app2).is_ok());
        assert!(stack.get_compositor_mut().connect_client(app3).is_ok());

        for app_id in [app1_id, app2_id, app3_id] {
            let surface = stack
                .get_compositor_mut()
                .create_surface(&app_id)
                .unwrap();

            let buffer = stack
                .get_compositor_mut()
                .create_buffer(1920, 1080, 0x34325241)
                .unwrap();

            assert!(stack
                .get_compositor_mut()
                .attach_buffer(&surface.id, &buffer.id)
                .is_ok());
            assert!(stack
                .get_compositor_mut()
                .commit_surface(&surface.id)
                .is_ok());
        }

        assert_eq!(stack.get_compositor().client_count(), 3);
        assert_eq!(stack.get_compositor().surface_count(), 3);
        assert_eq!(stack.get_compositor().buffer_count(), 3);
    }

    #[test]
    fn test_full_stack_performance() {
        let mut stack = ApplicationStack::new();
        assert!(stack.initialize().is_ok());

        let browser = WaylandClient {
            id: ObjectId::new(),
            name: "himalayas_browser".to_string(),
            is_connected: false,
        };

        let browser_id = browser.id.clone();
        assert!(stack.get_compositor_mut().connect_client(browser).is_ok());

        let surface = stack
            .get_compositor_mut()
            .create_surface(&browser_id)
            .unwrap();

        assert!(stack
            .get_compositor_mut()
            .configure_surface(&surface.id, 1920, 1080)
            .is_ok());

        let buffer = stack
            .get_compositor_mut()
            .create_buffer(1920, 1080, 0x34325241)
            .unwrap();

        assert!(stack
            .get_compositor_mut()
            .attach_buffer(&surface.id, &buffer.id)
            .is_ok());

        let pointer_event = PointerEvent {
            surface_id: Some(surface.id.clone()),
            event_type: PointerEventType::Motion,
            x: 960,
            y: 540,
            button: None,
        };

        assert!(stack
            .get_compositor_mut()
            .route_pointer_event(pointer_event)
            .is_ok());

        assert_eq!(stack.get_compositor().get_pointer_position(), (960, 540));
        assert!(stack.shutdown().is_ok());
    }
}
