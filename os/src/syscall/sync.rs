

use crate::sync::{Condvar, Mutex, MutexBlocking, MutexSpin, Semaphore};
use crate::task::{block_current_and_run_next, current_process, current_task};
use crate::timer::{add_timer, get_time_ms};
use alloc::sync::Arc;
/// sleep syscall
pub fn sys_sleep(ms: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_sleep",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let expire_ms = get_time_ms() + ms;
    let task = current_task().unwrap();
    add_timer(expire_ms, task);
    block_current_and_run_next();
    0
}
/// mutex create syscall
pub fn sys_mutex_create(blocking: bool) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mutex: Option<Arc<dyn Mutex>> = if !blocking {
        Some(Arc::new(MutexSpin::new()))
    } else {
        Some(Arc::new(MutexBlocking::new()))
    };
    let mut process_inner = process.inner_exclusive_access();
    if let Some(id) = process_inner
        .mutex_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.mutex_list[id] = mutex;
        if process_inner.deadlock_detect{
            debug!("create work[{}] = {}",id,process_inner.work[id]+1);
            process_inner.work.push(1); // 对于mutex，id就是push的次数
        }
        id as isize
    } else {
        process_inner.mutex_list.push(mutex);
        if process_inner.deadlock_detect{
            debug!("create work[{}] = {}",process_inner.mutex_list.len() as usize - 1,1);
            process_inner.work.push(1); // 对于mutex，id就是push的次数
        }
        process_inner.mutex_list.len() as isize - 1
    }
}
/// mutex lock syscall
pub fn sys_mutex_lock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_lock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    // 银行家算法检查资源
    if process_inner.deadlock_detect == true{
        if process_inner.work[mutex_id] == 0{
            debug!("bu zu,work[{mutex_id}] == 0");
            drop(process_inner);
            drop(process);
            -0xdead
        }else {
            debug!("ok,work[{}] == {}",mutex_id,process_inner.work[mutex_id]);
            process_inner.work[mutex_id] -= 1;
            drop(process_inner);
            drop(process);
            mutex.lock();
            0
        }
    }else {
        drop(process_inner);
        drop(process);
        mutex.lock();
        0
    }
}
/// mutex unlock syscall
pub fn sys_mutex_unlock(mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_mutex_unlock",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    if process_inner.deadlock_detect == true{
        process_inner.work[mutex_id] -= 1;
        let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
        process_inner.allocated[tid][mutex_id] -= 1;
    }
    drop(process_inner);
    drop(process);
    mutex.unlock();
    0
}
/// semaphore create syscall
pub fn sys_semaphore_create(res_count: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .semaphore_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.semaphore_list[id] = Some(Arc::new(Semaphore::new(res_count)));
        debug!("create {} count is {}",id,res_count);
        process_inner.work.push(res_count as u32);
        id
    } else {
        process_inner
            .semaphore_list
            .push(Some(Arc::new(Semaphore::new(res_count))));
        debug!("create {} count is {}",process_inner.semaphore_list.len() - 1,res_count);
        process_inner.work.push(res_count as u32);
        process_inner.semaphore_list.len() - 1
    };
    id as isize
}
/// semaphore up syscall
pub fn sys_semaphore_up(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_up",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    if process_inner.deadlock_detect == true{
        debug!("release {}, now work[{}] == {}",sem_id,sem_id,process_inner.work[sem_id]+1);
        let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
        process_inner.work[sem_id] += 1;
        process_inner.allocated[tid][sem_id] -= 1;
    }
    drop(process_inner);
    sem.up();
    0
}
/// semaphore down syscall
pub fn sys_semaphore_down(sem_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_semaphore_down",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let sem = Arc::clone(process_inner.semaphore_list[sem_id].as_ref().unwrap());
    
    let tid = current_task().unwrap().inner_exclusive_access().res.as_ref().unwrap().tid;
    if process_inner.deadlock_detect == true{
        process_inner.need[tid][sem_id] += 1; //somls 增加需求
        if process_inner.work[sem_id] >= process_inner.need[tid][sem_id]{ //somls 资源充足，分配资源
            debug!("down ok work[{sem_id}] == {}",process_inner.work[sem_id]);
            process_inner.work[sem_id] -= process_inner.need[tid][sem_id]; //somls 可分配资源增加
            process_inner.allocated[tid][sem_id] += process_inner.need[tid][sem_id]; // somls 已分配资源增加
            process_inner.need[tid][sem_id] = 0; // 还需要分配资源减少
            drop(process_inner);
            drop(process);
            sem.down();
            0
        }
        else {// somls 资源不够分配，寻找其他线程执行，如果找不到就返回不可分配
            debug!("bu zu,work[{sem_id}] == 0");
            for td in 1..process_inner.tasks.len(){ // 不需要从0开始，只需要比较1-n个线程
                if td == tid || process_inner.finish[td] == true{
                    continue;
                }
                let mut available = true;
                for (work_idx,work_num) in process_inner.work.iter().enumerate(){
                    debug!("td = {},sem_id = {},w = {}, n = {}",td,work_idx,work_num,process_inner.need[td][work_idx]);
                    if *work_num < process_inner.need[td][work_idx]{
                        available = false;
                        break;
                    }
                }
                if available == true{ // 当前线程td的需要的资源都可以正常分配
                    debug!("drop now tid = {}, available to run = {}",tid,td);
                    drop(process_inner);
                    drop(process);
                    sem.down();
                    return 0;
                }
            }
            // 当前存在死锁，取消锁分配
            debug!("exsist deadlock , don't set {} down",sem_id);
            drop(process_inner);
            drop(process);
            -0xdead
        }
    }else { // somls 不需要检验    
        drop(process_inner);
        sem.down();
        0
    }
}
/// condvar create syscall
pub fn sys_condvar_create() -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_create",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let mut process_inner = process.inner_exclusive_access();
    let id = if let Some(id) = process_inner
        .condvar_list
        .iter()
        .enumerate()
        .find(|(_, item)| item.is_none())
        .map(|(id, _)| id)
    {
        process_inner.condvar_list[id] = Some(Arc::new(Condvar::new()));
        id
    } else {
        process_inner
            .condvar_list
            .push(Some(Arc::new(Condvar::new())));
        process_inner.condvar_list.len() - 1
    };
    id as isize
}
/// condvar signal syscall
pub fn sys_condvar_signal(condvar_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_signal",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    drop(process_inner);
    condvar.signal();
    0
}
/// condvar wait syscall
pub fn sys_condvar_wait(condvar_id: usize, mutex_id: usize) -> isize {
    trace!(
        "kernel:pid[{}] tid[{}] sys_condvar_wait",
        current_task().unwrap().process.upgrade().unwrap().getpid(),
        current_task()
            .unwrap()
            .inner_exclusive_access()
            .res
            .as_ref()
            .unwrap()
            .tid
    );
    let process = current_process();
    let process_inner = process.inner_exclusive_access();
    let condvar = Arc::clone(process_inner.condvar_list[condvar_id].as_ref().unwrap());
    let mutex = Arc::clone(process_inner.mutex_list[mutex_id].as_ref().unwrap());
    drop(process_inner);
    condvar.wait(mutex);
    0
}
/// enable deadlock detection syscall
///
/// YOUR JOB: Implement deadlock detection, but might not all in this syscall
pub fn sys_enable_deadlock_detect(_enabled: usize) -> isize {
    trace!("kernel: sys_enable_deadlock_detect NOT IMPLEMENTED");
    if _enabled == 1{
        current_process().inner_exclusive_access().deadlock_detect = true;
    }else {
        current_process().inner_exclusive_access().deadlock_detect = false;
    }
    0
}
