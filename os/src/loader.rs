use core::ffi::CStr;
use lazy_static::lazy_static;
use log::debug;
use alloc::vec::Vec;


lazy_static! {
    static ref APP_NAMES: Vec<&'static str> = {
        let num_app = get_num_app();
        extern "C" { fn _app_names(); }
        let mut start = _app_names as *const i8;
        let mut v = Vec::new();
        for _ in 0..num_app {
            let str = unsafe { CStr::from_ptr(start) }.to_str().unwrap();
            v.push(str);
            unsafe { start = start.add(str.len() + 1); }
        }
        v
    };
}


pub fn get_num_app() -> usize {
    extern "C" { fn _num_app(); }
    unsafe {
        (_num_app as *const usize).read_volatile()
    }
}

pub fn get_app_data_by_name(name: &str) -> Option<&'static [u8]> {
    let num_app = get_num_app();
    (0..num_app)
        .find(|&i| APP_NAMES[i] == name)
        .map(|i| get_app_data(i))
}

pub fn list_apps() {
    println!("/**** APPS ****");
    for app in APP_NAMES.iter() {
        println!("{}", app);
    }
    println!("**************/");
}

pub fn get_app_data(app_id: usize) -> &'static [u8] {
    debug!("get data in appid {}", app_id);
    extern "C" { fn _num_app(); }
    let app_ptr = _num_app as *const usize;
    let num_app = unsafe {app_ptr.read_volatile()};
    let app_start = unsafe {
        core::slice::from_raw_parts(app_ptr.add(1), num_app + 1)
    };
    assert!(app_id < num_app, "Invalid app_id {app_id}! ");
    unsafe {
        core::slice::from_raw_parts(
            app_start[app_id] as *const u8,
            app_start[app_id + 1] - app_start[app_id]
        )
    }
}
