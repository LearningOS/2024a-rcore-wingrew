//! Process management syscalls
// use core::{ops::Add, ptr};

use crate::{
    mm::{self, frame_alloc, page_table::PTEFlags, translated_byte_buffer, VPNRange, VirtAddr}, task::{change_program_brk, current_user_token, exit_current_and_run_next, map_one, show_info, suspend_current_and_run_next, unmap_one, TaskInfo}, timer::{get_time_ms, get_time_us}
};
use core::{mem::size_of, ptr::{copy_nonoverlapping, write_unaligned}};
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}



/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let tv_sec = us / 1_000_000;
    let tv_usec = us % 1_000_000;
    let mut ts = translated_byte_buffer(current_user_token(), _ts as *const u8, core::mem::size_of::<TimeVal>());

    unsafe {
        // 获取缓冲区的原始指针
        let ptr = ts[0].as_mut_ptr() as *mut i64;

        // 将 tv_sec 写入偏移 0 的位置
        write_unaligned(ptr, tv_sec as i64);

        // 将 tv_usec 写入偏移 8 的位置
        write_unaligned(ptr.add(1), tv_usec as i64);        
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let mut temp = show_info();
    temp.time = get_time_ms() - temp.time;
    let mut ti = translated_byte_buffer(current_user_token(), _ti as *const u8, core::mem::size_of::<TaskInfo>());
    let total_bytes = size_of::<TaskInfo>();
    let mut bytes_written = 0;
    for slice in ti.iter_mut(){
        let slice_len = slice.len();
        let mut offset = 0;
        while offset < slice_len && bytes_written < total_bytes{
            unsafe {
                let to_write = (total_bytes - bytes_written).min(slice_len - offset);
                let ptr = slice.as_mut_ptr().add(offset);
                let struct_ptr = &temp as *const TaskInfo as *const u8;
                copy_nonoverlapping(struct_ptr.add(bytes_written), ptr, to_write);
            }
            offset += slice_len;
            bytes_written += slice_len;
        }
        if bytes_written >= total_bytes {
            break;
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _start % 4096 != 0 || _port & !0x7 != 0 || _port & 0x7 == 0{
        return -1;
    }

    let start_va = VirtAddr::from(_start).floor();
    let end_va = VirtAddr::from(_start + _len).ceil();
    let vir = VPNRange::new(start_va, end_va);
    let port = (_port as u8) << 5 >> 4;
    let mut flag = PTEFlags::U;
    flag |= PTEFlags::from_bits(port).unwrap();
    for vpn in vir{
        let page_table = mm::page_table::PageTable::from_token(current_user_token());
        let frame = frame_alloc().unwrap();
        let result = page_table.translate(vpn);
        match result{
            Some(pey) => {
                if !pey.is_valid(){
                    map_one(vpn, frame.ppn, flag);
                }else{
                    
                    return -1;
                }
            },
            None => {
                map_one(vpn, frame.ppn, flag);
            },
        }
    }
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if _start % 4096 != 0{
        return -1;
    }
    let start_va = VirtAddr::from(_start).floor();
    let end_va = VirtAddr::from(_start + _len).ceil();
    let vir = VPNRange::new(start_va, end_va);    
    for vpn in vir{
        let page_table = mm::page_table::PageTable::from_token(current_user_token());
        let result = page_table.translate(vpn);
        match result{
            Some(pey) => {
                if !pey.is_valid(){
                    return -1;
                }
                unmap_one(vpn);
            },
            None => return -1,
        }
    }
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
