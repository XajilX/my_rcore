use crate::drivers::INPUT_DEV;

pub fn sys_input_event() -> isize {
    let ret = INPUT_DEV.then(|devs| {
        for dev in devs {
            if !dev.is_empty() {
                return dev.read_event();
            }
        }
        0
    });
    ret as isize
}
