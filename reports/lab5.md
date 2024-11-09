# 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
    无
2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
    无
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
# 实现的功能
ProcessControlInner添加银行家算法资源
```
pub deadlock_detect: bool,
/// 可以分配的资源数量
pub work: Vec<u32>, 
/// 已经分配的资源数量
pub allocated: Vec<Vec<u32>>,
/// 线程需要的资源数量
pub need: Vec<Vec<u32>>,
/// 线程完成情况
pub finish: Vec<bool>,
```
need数组和allocated数组初始化时设置为[[0;10];10]，初始化大小为10x10的全0矩阵。
每添加一个信号量，将work对应的信号量值设置为count
sem_up时根据tid释放1个已分配的sem_id，work对应增加，然后才执行信号量释放代码。
sem_down时，比较是否能分配，如果当前资源不足，则查找是否有线程未完成且need < work，如果有则不存在死锁，可以执行信号量lock，如果不存在则拒绝lock，返回-0xdead。
当线程执行exit时，将finish设置为true。

# 简答作业
1. 在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出， 此时需要结束该进程管理的所有线程并回收其资源。 - 需要回收的资源有哪些？ - 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？
    需要回收的资源：线程栈空间
    TaskControlBlock（TCB）引用位置及回收情况：线程调度队列中的引用、其他线程内部对自身 TCB 的引用、资源管理模块对 TCB 的引用

2. 对比以下两种 Mutex.unlock 的实现，二者有什么区别？这些区别可能会导致什么问题？
    ```rust
    impl Mutex for Mutex1 {
        fn unlock(&self) {
            let mut mutex_inner = self.inner.exclusive_access();
            assert!(mutex_inner.locked);
            mutex_inner.locked = false;
            if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
                add_task(waking_task);
            }
        }
    }

    impl Mutex for Mutex2 {
        fn unlock(&self) {
            let mut mutex_inner = self.inner.exclusive_access();
            assert!(mutex_inner.locked);
            if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
                add_task(waking_task);
            } else {
                mutex_inner.locked = false;
            }
        }
    }
    ```
    Mutex1先释放了锁，如果有task在等待，则将它加入任务队列
    Mutex2先判断是否有task在等待，如果有则不释放锁，直接释放等待的task。当没有等待的task时，释放锁。
    Mutex1释放锁还未将task加入队列前，如果时间片到，则新的线程lock锁时会优先访问内部资源。
    Mutex2则需要将等待锁的队列中所有task都执行完毕后，才释放锁，这样避免了后来的线程先访问临界区。
    相比较而言，Mutex1会存在饥饿问题，先申请锁的线程后访问临界区。
