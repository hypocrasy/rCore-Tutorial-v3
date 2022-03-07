use core::iter::Map;

use riscv::paging::PageTable;
use riscv::register::fcsr::Flag;

use crate::config::PAGE_SIZE;
use crate::console::print;
use crate::mm::{translated_byte_buffer,new_pagetable, VirtAddr,MapType,MapPermission, MemorySet,VirtPageNum};
//use crate::mm::page_table::new_pagetable;
use crate::task::{current_user_token, TaskManager,TASK_MANAGER, current_user_insert,current_user_free,find_vpn};
const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        },
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
pub fn sys_mmap(start:usize,len:usize,port:usize) -> isize{
    //let usertoken=current_user_token();
    //let userpt=new_pagetable(usertoken);
    if (start%PAGE_SIZE!=0) || (port & !0x7 != 0) || (port & 0x7 == 0) {
        return -1;
    }
    let mut length = len;
    if len % 4096 != 0 {
        length = len + (4096 - len % 4096);
    }
    let start_vpn=start/4096;
    let end_vpn=(start + length) / 4096;
    //println!("map start:{},end:{}",start_vpn.floor().0,end_vpn.ceil().0);
    for vpn in(start_vpn..end_vpn){
        match find_vpn(VirtPageNum::from(vpn)){
            false =>{continue},
            true =>{
                println!("Page exist");
                return -1
            },
        }
    }

    let permission = match port {
        1 => MapPermission::U | MapPermission::R,
        2 => return -1,
        3 => MapPermission::U | MapPermission::R | MapPermission::W,
        4 => MapPermission::U | MapPermission::X,
        5 => MapPermission::U | MapPermission::R | MapPermission::X,
        6 => MapPermission::U | MapPermission::X | MapPermission::W,
        _ => MapPermission::U | MapPermission::R | MapPermission::W | MapPermission::X,
    };
    current_user_insert(VirtAddr::from(start), VirtAddr::from(start+length), permission);
    0
 
}
pub fn sys_munmap(start:usize,len:usize)->isize{
   // println!("enter sysmunmap");
    let usertoken=current_user_token();
    let userpt=new_pagetable(usertoken);
    let mut length = len;
    if len % 4096 != 0 {
        length = len + (4096 - len % 4096);
    }
    let start_vpn=start/4096;
    let end_vpn=(start + length) / 4096;
    if(start%PAGE_SIZE!=0){return -1;}
    //println!("unmap start:{},end:{}",start_vpn.floor().0,end_vpn.ceil().0-1);
    for vpn in(start_vpn..end_vpn){
        match find_vpn(VirtPageNum(vpn)){
            false =>{
                println!("Page not found");
                return -1
            },
            true =>{continue},
        }
    }
    current_user_free(VirtPageNum::from(start_vpn), VirtPageNum::from(end_vpn));
    0
}