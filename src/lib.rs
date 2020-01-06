mod vigem_api_gen;
pub use vigem_api_gen::XUSB_REPORT as XUsbReport;
pub use vigem_api_gen::{DS4_BUTTONS, XUSB_BUTTON};
use vigem_api_gen::{PVIGEM_CLIENT, PVIGEM_TARGET, PVOID, UCHAR};

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
            Ok(ViGEm {
                client,
                targets: Vec::new(),
            })
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

    pub fn target_x360_update(&self, report: XUsbReport) {
        unsafe {
            for (target, target_type) in self.targets.iter() {
                if let TargetType::X360 = target_type {
                    vigem_api_gen::vigem_target_x360_update(self.client, *target, report);
                };
            }
        };
    }

    /*pub fn register_notification(&self, notification: vigem_api_gen::EVT_VIGEM_X360_NOTIFICATION, user_data: vigem_api_gen::PVOID) {
        unsafe {
            for (target, target_type) in self.targets.iter() {
               if let TargetType::X360 = target_type {
                    vigem_api_gen::vigem_target_x360_register_notification(self.client, *target, notification, user_data);
               };
            };
        };
    }*/
    pub fn register_x360_notification<F>(&self, notification: F)
    where
        F: FnMut(UCHAR, UCHAR, UCHAR),
    {
        let data = Box::into_raw(Box::new(notification));
        unsafe {
            for (target, target_type) in self.targets.iter() {
                if let TargetType::X360 = target_type {
                    vigem_api_gen::vigem_target_x360_register_notification(
                        self.client,
                        *target,
                        Some(call_closure::<F>),
                        data as _,
                    );
                }
            }
        }
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

unsafe extern "C" fn call_closure<F>(
    _client: PVIGEM_CLIENT,
    _target: PVIGEM_TARGET,
    large_motor: UCHAR,
    small_motor: UCHAR,
    led_number: UCHAR,
    user_data: PVOID,
) where
    F: FnMut(UCHAR, UCHAR, UCHAR),
{
    let callback_ptr = user_data as *mut F;
    let callback = &mut *callback_ptr;
    callback(large_motor, small_motor, led_number);
}
