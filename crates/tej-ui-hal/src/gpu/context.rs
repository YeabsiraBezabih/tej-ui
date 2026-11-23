//! GPU device and queue initialization, adapter capabilities, and shared context.

use crate::gpu::types::{BackendConfig, GpuError};
use std::sync::Arc;

/// Queryable hardware adapter capabilities and runtime limits.
#[derive(Debug, Clone)]
pub struct AdapterCapabilities {
    pub adapter_name: String,
    pub backend: wgpu::Backend,
    pub max_texture_dimension_2d: u32,
    pub max_storage_buffer_binding_size: u32,
    pub is_software_fallback: bool,
}

/// Authoritative GPU execution context owning shared `wgpu::Device` and `wgpu::Queue`.
///
/// Designed for zero-copy interop: `tej-3d` and custom views borrow or clone `Arc` handles
/// directly from this context, enabling GPU memory sharing without CPU readback.
#[derive(Debug, Clone)]
pub struct GpuContext {
    instance: wgpu::Instance,
    adapter: Arc<wgpu::Adapter>,
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    capabilities: AdapterCapabilities,
}

impl GpuContext {
    /// Create a new `wgpu::Instance` with default backends.
    #[must_use]
    pub fn create_instance() -> wgpu::Instance {
        wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::default(),
            backend_options: wgpu::BackendOptions::default(),
        })
    }

    /// Synchronously construct a headless GPU context (suitable for tests and offscreen rendering).
    pub fn new_headless(config: &BackendConfig) -> Result<Self, GpuError> {
        pollster::block_on(Self::new_async(config, None))
    }

    /// Asynchronously initialize a GPU context with optional surface compatibility.
    pub async fn new_async(
        config: &BackendConfig,
        compatible_surface: Option<&wgpu::Surface<'_>>,
    ) -> Result<Self, GpuError> {
        let instance = Self::create_instance();

        let adapter_opt = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: config.power_preference,
                compatible_surface,
                force_fallback_adapter: false,
            })
            .await;

        let adapter = match adapter_opt {
            Some(a) => a,
            None if config.allow_software_fallback => instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: config.power_preference,
                    compatible_surface,
                    force_fallback_adapter: true,
                })
                .await
                .ok_or(GpuError::AdapterNotFound)?,
            None => return Err(GpuError::AdapterNotFound),
        };

        let adapter_info = adapter.get_info();
        let adapter_limits = adapter.limits();

        let capabilities = AdapterCapabilities {
            adapter_name: adapter_info.name,
            backend: adapter_info.backend,
            max_texture_dimension_2d: adapter_limits.max_texture_dimension_2d,
            max_storage_buffer_binding_size: adapter_limits.max_storage_buffer_binding_size,
            is_software_fallback: adapter_info.device_type == wgpu::DeviceType::Cpu,
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("tej_gpu_device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|e| GpuError::DeviceRequestFailed(e.to_string()))?;

        Ok(Self {
            instance,
            adapter: Arc::new(adapter),
            device: Arc::new(device),
            queue: Arc::new(queue),
            capabilities,
        })
    }

    /// Returns reference to the WGPU instance.
    #[must_use]
    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    /// Returns reference to the selected physical adapter.
    #[must_use]
    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    /// Returns reference to the logical device.
    #[must_use]
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Clones an `Arc<wgpu::Device>` handle for cross-subsystem zero-copy sharing.
    #[must_use]
    pub fn device_arc(&self) -> Arc<wgpu::Device> {
        Arc::clone(&self.device)
    }

    /// Returns reference to the command queue.
    #[must_use]
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Clones an `Arc<wgpu::Queue>` handle for cross-subsystem zero-copy sharing.
    #[must_use]
    pub fn queue_arc(&self) -> Arc<wgpu::Queue> {
        Arc::clone(&self.queue)
    }

    /// Returns the queried capabilities of the underlying adapter.
    #[must_use]
    pub fn capabilities(&self) -> &AdapterCapabilities {
        &self.capabilities
    }
}
