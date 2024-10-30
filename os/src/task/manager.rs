//!Implementation of [`TaskManager`]

use core::usize::MAX;

use super::TaskControlBlock;
use crate::sync::UPSafeCell;
// use alloc::collections::binary_heap::BinaryHeap;
// use alloc::collections::VecDeque;
use alloc::{sync::Arc, vec::Vec};
use lazy_static::*;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_vector: Vec<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_vector: Vec::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        // self.ready_heap.push(task);
        self.ready_vector.push(task);
    }
    /// Take a process out of the ready queue
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        // self.ready_heap.pop()
        if self.ready_vector.len() == 0{
            return None;
        }
        let mut min_stride = MAX;
        let mut min_stride_task_idx = 0;
        for i in 0..self.ready_vector.len(){
            let current_stride = self.ready_vector[i].inner_exclusive_access().task_info.stride;
            if current_stride < min_stride{
                min_stride = current_stride;
                min_stride_task_idx = i;
            }
        }
        Some(self.ready_vector.remove(min_stride_task_idx))
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
