const FD_STDOUT: usize = 1;
use riscv::register::sscratch::read;

use crate::{task::{TASK_MANAGER, get_current_app}, config::{APP_SIZE_LIMIT, APP_BASE_ADDRESS}, timer::get_time_us, loader::get_user_sp};
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    //unsafe{println!("{}",fd)};
    match fd {
        FD_STDOUT => {
            let current_app =get_current_app();
            let sscratch=read();
            let start=(current_app) as usize*APP_SIZE_LIMIT+APP_BASE_ADDRESS;
            //unsafe{println!("{} {} {} {} {} {}",buf as usize,len,start,start+APP_SIZE_LIMIT,sscratch,get_user_sp())};
            if( (buf as usize>=start&&buf as usize+len<=start+APP_SIZE_LIMIT) || (buf as usize>=sscratch&&buf as usize+len<=get_user_sp())){
                let slice = unsafe { core::slice::from_raw_parts(buf, len) };
                let str = core::str::from_utf8(slice).unwrap();
                print!("{}", str);
                len as isize
            }
            else {
                unsafe{println!("address fault!")};
                -1
            }
        },
        _ => {
            unsafe{println!("Unsupported fd in sys_write!")};
            -1
        }
    }
}