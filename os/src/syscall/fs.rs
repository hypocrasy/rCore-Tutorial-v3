const FD_STDOUT: usize = 1;
use core::{arch::asm, borrow::Borrow};
use crate::batch::{APP_MANAGER, USER_STACK};
use crate::batch::{USER_STACK_SIZE,APP_SIZE_LIMIT,APP_BASE_ADDRESS};
use crate::console::print;
use riscv::register::sscratch::read;
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    //let mut sscratch: usize;
    // unsafe{asm!("mv {}, sscratch", out(reg) sscratch);}   
    match fd {
        FD_STDOUT => {
        
            let user_sscratch=read();
            let mut app_manager = APP_MANAGER.exclusive_access();
            let current_app = app_manager.get_current_app();
            let current_app_start=app_manager.get_current_app_start();
            let current_app_end=app_manager.get_next_app_start();
            unsafe {println!("#{:#x} {:#x} #", buf as usize , USER_STACK.get_sp() - USER_STACK_SIZE);}
            //println!("write: {:#x} - {:#x}",buf as usize,buf as usize+len);
            //println!("app: {} is {:#x} - {:#x}",current_app,current_app_start,current_app_end);
            //println!("stack:{:#x} - {:#x}\n",user_sscratch,(USER_STACK.data.as_ptr() as usize+USER_STACK_SIZE));
            let addr=buf as usize;
            if (((buf as usize)  >= USER_STACK.get_sp() - USER_STACK_SIZE) && ((buf as usize) + len <= USER_STACK.get_sp())) 
            || (((buf as usize) + len <= APP_SIZE_LIMIT + APP_BASE_ADDRESS) && ((buf as usize) >= APP_BASE_ADDRESS)){
            //if(true){

             let slice = unsafe { core::slice::from_raw_parts(buf, len) };
             let str = core::str::from_utf8(slice).unwrap();
             print!("{}", str);
             len as isize
            }
         else {
                //println!("addr fault");
                -1
            }
        },
         _ => {
        println!("Unsupported fd in sys_write!\n");
            -1
        }
    }
}