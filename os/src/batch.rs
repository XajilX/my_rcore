const MAX_APP_NUM: usize = 100;
struct AppMan {
    num_app: usize,
    curr_app: usize,
    app_start: [usize; MAX_APP_NUM + 1]
}

use lazy_static::lazy_static;
use crate::uthr::UThrCell;
use core::slice::from_raw_parts;
lazy_static! {
    static ref APP_MAN: UThrCell<AppMan> = unsafe {
        extern "C" { fn _num_app(); }
        let num_app_ptr = _num_app as usize as * const usize;
        let num_app = num_app_ptr.read_volatile();
        let mut app_start = [0usize; MAX_APP_NUM + 1];
        let app_start_raw = from_raw_parts(
            num_app_ptr.add(1), num_app + 1
        );
        app_start[..=num_app].copy_from_slice(app_start_raw);
        UThrCell::new(AppMan {
            num_app,
            curr_app: 0,
            app_start
        })
    };
}