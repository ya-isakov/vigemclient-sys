mod vigem_api_gen;
pub use vigem_api_gen::XUSB_REPORT as XUsbReport;
pub use vigem_api_gen::{DS4_BUTTONS, XUSB_BUTTON};
use vigem_api_gen::{PVIGEM_CLIENT, PVIGEM_TARGET, PVOID, UCHAR};

pub enum TargetType {
    X360,
    Ds4,
}

pub struct ViGEm {
    client: PVIGEM_CLIENT,
    targets: Vec<(PVIGEM_TARGET, TargetType, Option<PVOID>)>,
}

type Callback = dyn FnMut(UCHAR, UCHAR, UCHAR) + 'static;
type BCallback = Box<Callback>;

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
            self.targets.push((target, target_type, None));
        }
        Ok(())
    }

    pub fn target_x360_update(&self, report: XUsbReport) {
        unsafe {
            for (target, target_type, _) in self.targets.iter() {
                if let TargetType::X360 = target_type {
                    vigem_api_gen::vigem_target_x360_update(self.client, *target, report);
                };
            }
        };
    }

    pub fn register_x360_notification<F>(&mut self, notification: F)
    where
        F: FnMut(UCHAR, UCHAR, UCHAR) + 'static,
    {
        let cb: Box<BCallback> = Box::new(Box::new(notification));
        let data_ptr = Box::into_raw(cb) as PVOID;
        unsafe {
            for (target, target_type, notif) in self.targets.iter_mut() {
                if let TargetType::X360 = target_type {
                    vigem_api_gen::vigem_target_x360_register_notification(
                        self.client,
                        *target,
                        Some(call_closure),
                        data_ptr,
                    );
                    *notif = Some(data_ptr);
                }
            }
        }
    }
}

unsafe fn drop_box(user_data: PVOID) {
    // I hope that I correctly clean this...
    let _: Box<BCallback> = Box::from_raw(user_data as *mut _);
}

//#[cfg(target_os = "windows")]
impl Drop for ViGEm {
    fn drop(&mut self) {
        unsafe {
            for (target, target_type, notif) in self.targets.iter_mut() {
                if let TargetType::X360 = target_type {
                    if let Some(n) = *notif {
                        println!("Releasing notification");
                        vigem_api_gen::vigem_target_x360_unregister_notification(*target);
                        drop_box(n);
                        *notif = None;
                    }
                    let _res = vigem_api_gen::vigem_target_remove(self.client, *target);
                }
            }
        }
    }
}

unsafe extern "C" fn call_closure(
    _client: PVIGEM_CLIENT,
    _target: PVIGEM_TARGET,
    large_motor: UCHAR,
    small_motor: UCHAR,
    led_number: UCHAR,
    user_data: PVOID,
)
{
    // Black magic, not sure, what happens here, but clippy gave this as a replacement for mem::transmute
    let callback: &mut Callback = &mut *(user_data as *mut BCallback);
    callback(large_motor, small_motor, led_number);
}
