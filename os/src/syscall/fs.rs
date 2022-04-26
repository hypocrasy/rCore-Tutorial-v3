

use core::ptr;


use crate::mm::{
    UserBuffer,
    translated_byte_buffer,
    translated_refmut,
    translated_str, translate_va,
};
use crate::task::{current_user_token, current_task};
use crate::fs::{make_pipe, OpenFlags, open_file, OSInode,find_app,root_insert_dirent,root_delete_dirent, File, StatMode, root_get_nlink_by_id};
use alloc::sync::Arc;
use easy_fs::{DirEntry, DiskInodeType};
use k210_hal::cache::Uncache;
use crate::fs::Stat;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    //println!("{}",path);
    if let Some(inode) = open_file(
        path.as_str(),
        OpenFlags::from_bits(flags).unwrap()
    ) {
        //println!("new inodeid:{}",inode.get_inode_id());
        //println!("new nlink:{}",root_get_nlink_by_id(inode.get_inode_id() as u32) );
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let mut inner = task.inner_exclusive_access();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}

pub fn sys_dup(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    let new_fd = inner.alloc_fd();
    inner.fd_table[new_fd] = Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));
    new_fd as isize
}
pub fn sys_unlinkat(path: *const u8) ->i32{
    let token = current_user_token();
    let file=translated_str(token, path);
    root_delete_dirent(file.as_str())
}
pub fn sys_linkat(oldpath: *const u8,newpath: *const u8)->i32{
    let token = current_user_token();
    let oldfile=translated_str(token, oldpath);
    
    let newfile=translated_str(token, newpath);
    if newfile==oldfile  {
        return -1;
    }
    let inode=find_app(oldfile.as_str()).unwrap();
    let indoe_id=inode.get_inode_id();
    let dirent = DirEntry::new(newfile.as_str(), indoe_id);
    root_insert_dirent(dirent);
    
    0
}
pub fn sys_stat(fd: i32, _st:*mut Stat) -> isize{
    if fd<0 {
        return -1;
    }
    let token = current_user_token();
    let st=translated_refmut(token,_st);
    let task = current_task().unwrap();
    let  inner = task.inner_exclusive_access();
    if fd as usize >= inner.fd_table.len() {
        return -1;
    }
  
    if let Some(file) = &inner.fd_table[fd as usize] {
        let _file = file.clone();
        st.dev=0;
        st.ino=_file.get_inode_id();
     
        st.mode=_file.get_mode();
        st.nlink=root_get_nlink_by_id(_file.get_inode_id() as u32);
        

        }  
    0
}