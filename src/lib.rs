mod vigem_api_gen;
pub use vigem_api_gen::XUSB_REPORT as XUsbReport;
pub use vigem_api_gen::{DS4_BUTTONS, XUSB_BUTTON};

pub enum TargetType {
    X360,
    Ds4,
}

pub struct ViGEm {
    client: vigem_api_gen::PVIGEM_CLIENT,
    targets: Vec<(vigem_api_gen::PVIGEM_TARGET, TargetType)>,
}

impl ViGEm {
    pub fn new() -> Result<ViGEm, String> {
        unsafe {
            let client = vigem_api_gen::vigem_alloc();
            let res = vigem_api_gen::vigem_connect(client);
            if res != vigem_api_gen::VIGEM_ERROR::VIGEM_ERROR_NONE {
                return Err(format!("Error connecting to bus {:?}", res));
            }
            Ok(ViGEm { client, targets: Vec::new() })
        }
    }

    pub fn add_target(&mut self, target_type: TargetType) -> Result<(), String> {
        unsafe {
            let target = match target_type {
                TargetType::X360 => vigem_api_gen::vigem_target_x360_alloc(),
                TargetType::Ds4 => vigem_api_gen::vigem_target_ds4_alloc(),
            };
            let res = vigem_api_gen::vigem_target_add(self.client, target);
            if res != vigem_api_gen::VIGEM_ERROR::VIGEM_ERROR_NONE {
                return Err(format!("Error adding target: {:?}", res));
            }
            self.targets.push((target, target_type));
        }
        Ok(())
    }

    pub fn target_x360_update(&mut self, report: XUsbReport) {
        unsafe {
            for (target, target_type) in self.targets.iter_mut() {
                match target_type {
                    TargetType::X360 => vigem_api_gen::vigem_target_x360_update(self.client, *target, report),
                    _ => vigem_api_gen::VIGEM_ERROR::VIGEM_ERROR_NONE,
                };
            };
        };
    }
}

#[cfg(target_os = "windows")]
impl Drop for ViGEm {
    fn drop(&mut self) {
        unsafe {
            for (target, _) in self.targets.iter_mut() {
                let _res = vigem_api_gen::vigem_target_remove(self.client, *target);
            }
        }
    }
}
