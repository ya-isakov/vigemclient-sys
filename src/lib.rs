mod vigem_api_gen;
pub use vigem_api_gen::XUSB_REPORT as XUsbReport;
pub use vigem_api_gen::{DS4_BUTTONS, XUSB_BUTTON};
use vigem_api_gen::{LPVOID, PVIGEM_CLIENT, PVIGEM_TARGET, UCHAR};

pub enum TargetType {
    X360,
    Ds4,
}

pub struct ViGEm {
    client: PVIGEM_CLIENT,
    targets: Vec<ViGEmTarget>,
}

pub struct ViGEmTarget {
    target: PVIGEM_TARGET,
    target_type: TargetType,
    callback: Option<LPVOID>,
}

type Callback = dyn FnMut(UCHAR, UCHAR, UCHAR) + 'static;
type BCallback = Box<Callback>;

#[cfg(target_os = "windows")]
impl ViGEm {
    pub fn new() -> Result<ViGEm, vigem_api_gen::VIGEM_ERROR> {
        unsafe {
            let client = vigem_api_gen::vigem_alloc();
            match vigem_api_gen::vigem_connect(client) {
                vigem_api_gen::VIGEM_ERROR::VIGEM_ERROR_NONE => Ok(ViGEm {
                    client,
                    targets: Vec::new(),
                }),
                err => Err(err),
            }
        }
    }

    pub fn add_target(
        &mut self,
        target_type: TargetType,
    ) -> Result<(), vigem_api_gen::VIGEM_ERROR> {
        unsafe {
            let target = match target_type {
                TargetType::X360 => vigem_api_gen::vigem_target_x360_alloc(),
                TargetType::Ds4 => vigem_api_gen::vigem_target_ds4_alloc(),
            };
            match vigem_api_gen::vigem_target_add(self.client, target) {
                vigem_api_gen::VIGEM_ERROR::VIGEM_ERROR_NONE => {
                    let target = ViGEmTarget {
                        target,
                        target_type,
                        callback: None,
                    };
                    self.targets.push(target);
                    Ok(())
                }
                err => Err(err),
            }
        }
    }

    pub fn target_x360_update(&self, report: XUsbReport) -> Result<(), vigem_api_gen::VIGEM_ERROR> {
        unsafe {
            for target in self.targets.iter() {
                if let TargetType::X360 = target.target_type {
                    match vigem_api_gen::vigem_target_x360_update(
                        self.client,
                        target.target,
                        report,
                    ) {
                        vigem_api_gen::VIGEM_ERROR::VIGEM_ERROR_NONE => return Ok(()),
                        err => return Err(err),
                    }
                };
            }
        };
        Ok(())
    }

    pub fn register_x360_notification<F>(
        &mut self,
        notification: F,
    ) -> Result<(), vigem_api_gen::VIGEM_ERROR>
    where
        F: FnMut(UCHAR, UCHAR, UCHAR) + 'static,
    {
        let cb: Box<BCallback> = Box::new(Box::new(notification));
        let data_ptr = Box::into_raw(cb) as LPVOID;
        for current_target in self.targets.iter_mut() {
            if let TargetType::X360 = current_target.target_type {
                let res = unsafe {
                    vigem_api_gen::vigem_target_x360_register_notification(
                        self.client,
                        current_target.target,
                        Some(call_closure),
                        data_ptr,
                    )
                };
                match res {
                    vigem_api_gen::VIGEM_ERROR::VIGEM_ERROR_NONE => {
                        current_target.callback = Some(data_ptr);
                    }
                    err => return Err(err),
                };
            } else {
                return Err(vigem_api_gen::VIGEM_ERROR::VIGEM_ERROR_INVALID_PARAMETER);
            }
        }
        Ok(())
    }
}

unsafe fn drop_box(user_data: LPVOID) {
    // I hope that I correctly clean this...
    Box::from_raw(user_data as *mut _);
}

#[cfg(target_os = "windows")]
impl Drop for ViGEm {
    fn drop(&mut self) {
        unsafe {
            for t in self.targets.iter_mut() {
                if let TargetType::X360 = t.target_type {
                    if let Some(n) = t.callback {
                        vigem_api_gen::vigem_target_x360_unregister_notification(t.target);
                        drop_box(n);
                        t.callback = None;
                    }
                }
                vigem_api_gen::vigem_target_remove(self.client, t.target);
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
    user_data: LPVOID,
) {
    // Black magic. Not sure, what happens here, but clippy gave this as a replacement for mem::transmute
    let callback: &mut Callback = &mut *(user_data as *mut BCallback);
    callback(large_motor, small_motor, led_number);
}
