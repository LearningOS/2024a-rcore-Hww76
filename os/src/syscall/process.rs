//! Process management syscalls
use core::slice::from_raw_parts;

use crate::{
    config::{MAX_SYSCALL_NUM, PAGE_SIZE}, mm::{translated_byte_buffer, MapPermission, VirtPageNum}, task::{
    change_program_brk, current_user_token, exit_current_and_run_next, find_page_pte, get_current_task_first_time, get_current_task_syscall_times, insert_framed_area, suspend_current_and_run_next, unmap_in_memset, TaskStatus
    }, timer::get_time
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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
    let us = get_time();
    let buffers = &mut translated_byte_buffer(current_user_token(), _ts as *const u8, core::mem::size_of::<TimeVal>());
    let tmp_ts = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let tmp_ts_ptr = &tmp_ts as *const TimeVal as *const u8;
    let mut count = 0;
    for buffer in buffers.iter_mut() {
            unsafe { buffer.copy_from_slice(from_raw_parts(tmp_ts_ptr.add(count), buffer.len()) ); };
            count += buffer.len();
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let syscall_times = get_current_task_syscall_times();
    let start_time = get_current_task_first_time() - 2000;
    let now_time = get_time();
    let tmp_ti = TaskInfo{
        status: TaskStatus::Running,
        syscall_times: syscall_times,
        time: ((now_time-start_time)/1_000_000 & 0xffff) * 1000 + ((now_time-start_time)%1_000_000)/1000,
    };
    let buffers = &mut translated_byte_buffer(current_user_token(), _ti as *const u8, core::mem::size_of::<TaskInfo>());
    let tmp_ti_ptr = &tmp_ti as *const TaskInfo as *const u8;
    let mut count = 0;
    for buffer in buffers.iter_mut() {
            unsafe { buffer.copy_from_slice(from_raw_parts(tmp_ti_ptr.add(count), buffer.len()) ); };
            count += buffer.len();
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    let start_va = _start;
    let end_va = _start + _len; // end_va 是不包括的
    let mut result = 0;
    if _start % PAGE_SIZE != 0{
        result = -1;
    }else if _port & !0x7 != 0{
        result = -1;
    }else if _port & 0x7 == 0 {
        result = -1;
    }else if find_page_pte((start_va/PAGE_SIZE).into(),(end_va/PAGE_SIZE).into()) {
        result = -1;
    }else {
        let mut perm = MapPermission::U;
        if _port & 1 == 1{
            perm |= MapPermission::R;
        }
        if _port & 2 == 2{
            perm |= MapPermission::W;
        }
        if _port & 4 == 4 {
            perm |= MapPermission::X;
        }
        // println!("start_va = {:x},end_va = {:x}",start_va,end_va);
        insert_framed_area(start_va.into(), end_va.into(), perm); // 库函数中，end_va 不包括
    }
    result 
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let start_va = _start;
    let end_va = _start + _len - 1;
    let start_vpn:VirtPageNum = (start_va / PAGE_SIZE).into();
    let end_vpn:VirtPageNum = ((end_va + PAGE_SIZE - 1)/PAGE_SIZE).into();
    let mut vpn = start_vpn;
    while vpn <= end_vpn { // 存在页面没被映射
        if find_page_pte(vpn,vpn) == false {
            return -1;
        }
        vpn.0 += 1;
    }
    unmap_in_memset(start_va.into(),end_va.into());
    0
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    info!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
