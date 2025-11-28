use bluer::{Adapter, Address, Session};
use tokio::runtime::Runtime;
use anyhow::Result;

#[derive(Clone, Debug)]
pub struct BluetoothDevice {
    pub address: Address,
    pub name: String,
    pub icon: String,
    pub connected: bool,
    pub paired: bool,
}

impl BluetoothDevice {
    pub fn get_icon_name(&self) -> String {
        match self.icon.as_str() {
            "audio-card" | "audio-headset" => "audio-headphones".to_string(),
            "input-keyboard" => "input-keyboard".to_string(),
            "input-mouse" => "input-mouse".to_string(),
            "input-tablet" => "input-tablet".to_string(),
            "phone" => "phone".to_string(),
            "computer" => "computer".to_string(),
            "camera-photo" => "camera-photo".to_string(),
            "camera-video" => "camera-video".to_string(),
            _ => "bluetooth".to_string(),
        }
    }
}

pub struct BluetoothService {
    rt: Runtime,
    #[allow(dead_code)] // Kept to maintain session lifetime
    session: Session,
    adapter: Adapter,
}

impl BluetoothService {
    pub fn new() -> Result<Self> {
        let rt = Runtime::new()?;
        let (session, adapter) = rt.block_on(async {
            let session = Session::new().await?;
            let adapter = session.default_adapter().await?;
            adapter.set_powered(true).await?;
            Ok::<(Session, Adapter), anyhow::Error>((session, adapter))
        })?;

        Ok(Self {
            rt,
            session,
            adapter,
        })
    }

    pub fn is_powered(&self) -> bool {
        self.rt.block_on(async {
            self.adapter.is_powered().await.unwrap_or(false)
        })
    }

    pub fn power_on(&self) -> Result<()> {
        self.rt.block_on(async {
            self.adapter.set_powered(true).await?;
            Ok(())
        })
    }

    pub fn power_off(&self) -> Result<()> {
        self.rt.block_on(async {
            self.adapter.set_powered(false).await?;
            Ok(())
        })
    }

    pub fn get_devices(&self) -> Vec<BluetoothDevice> {
        self.rt.block_on(async {
            let mut devices = Vec::new();
            if let Ok(device_addresses) = self.adapter.device_addresses().await {
                for addr in device_addresses {
                    if let Ok(device) = self.adapter.device(addr) {
                        let name = device.name().await.unwrap_or(None).unwrap_or_else(|| "Unknown Device".to_string());
                        let icon = device.icon().await.unwrap_or(None).unwrap_or_else(|| "bluetooth".to_string());
                        let connected = device.is_connected().await.unwrap_or(false);
                        let paired = device.is_paired().await.unwrap_or(false);

                        devices.push(BluetoothDevice {
                            address: addr,
                            name,
                            icon,
                            connected,
                            paired,
                        });
                    }
                }
            }
            
            // Sort devices: specific device first, then connected, then paired, then name
            devices.sort_by(|a, b| {
                let a_special = a.name == "WF-C700";
                let b_special = b.name == "WF-C700";
                
                if a_special != b_special {
                    return b_special.cmp(&a_special);
                }
                
                if a.connected != b.connected {
                    return b.connected.cmp(&a.connected);
                }
                
                if a.paired != b.paired {
                    return b.paired.cmp(&a.paired);
                }
                
                a.name.cmp(&b.name)
            });

            devices
        })
    }

    pub fn connect_device(&self, address: Address) -> Result<()> {
        self.rt.block_on(async {
            let device = self.adapter.device(address)?;
            if !device.is_connected().await? {
                device.connect().await?;
            }
            Ok(())
        })
    }

    pub fn disconnect_device(&self, address: Address) -> Result<()> {
        self.rt.block_on(async {
            let device = self.adapter.device(address)?;
            if device.is_connected().await? {
                device.disconnect().await?;
            }
            Ok(())
        })
    }

    pub fn pair_device(&self, address: Address) -> Result<()> {
        self.rt.block_on(async {
            let device = self.adapter.device(address)?;
            if !device.is_paired().await? {
                device.pair().await?;
            }
            Ok(())
        })
    }
}