use sher_kernel::{SherKernel, KernelConfig};
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("Starting SHER Kernel...");

    let config = KernelConfig::default();
    let mut kernel = SherKernel::new(config)?;

    info!("Initializing kernel subsystems...");
    kernel.initialize().await?;

    info!("SHER Kernel initialized successfully");
    info!("System uptime: {:?}", kernel.uptime());

    kernel.shutdown().await?;
    Ok(())
}
