use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use std::error::Error;
use std::time::Duration;
use tokio::time;

pub async fn run_ble_scanner(tx: tokio::sync::mpsc::Sender<String>) -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = match adapters.into_iter().next() {
        Some(a) => a,
        None => {
            let _ = tx.send("BLE SYSTEM: No Adapter. Scanner Disabled.".to_string()).await;
            return Ok(());
        }
    };
    central.start_scan(ScanFilter::default()).await?;
    loop {
        for p in central.peripherals().await? {
            if let Some(props) = p.properties().await? {
                if let Some(name) = props.local_name {
                    if name.contains("Web4") {
                        let _ = tx.send(format!(">> BLE LINK: Found {}", name)).await;
                    }
                }
            }
        }
        time::sleep(Duration::from_secs(5)).await;
    }
}
