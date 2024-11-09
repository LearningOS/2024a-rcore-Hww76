# 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
    无
2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
    无
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
# 实现的功能
设置BigStride为0xff_0000，为TaskControlBlockInner增加特权级和步长字段，特权级初始为16，步长为0。在寻找下一个程序之后，运行下一个程序之前，更新其步长为BigStride/特权级。修改调度器结构体为小根堆，为TaskControlBlock添加比较Trait，每次选择stride最小的程序运行。
# 简答作业
stride 算法深入
stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。
* 实际情况是轮到 p1 执行吗？为什么？
    p2执行一个时间片后，p2.stride = 250 + 10 = 260
    但是由于stride是用8bit无符号数保存，p2的stride发生了溢出，变为了4
    因此p2执行一个时间片后，p2.stride < p1.stride
    由于溢出，导致下一个时间片仍然由p2执行
我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， 在不考虑溢出的情况下 , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。

* 为什么？尝试简单说明（不要求严格证明）。
    设特权级 i >= 2。每个进程的步长为 BigStride/i <= BigStride/2。
    当某个进程的优先级为2时，它的步长为BigStride/2,该进程每执行一次都会领先BigStride/2的步长，之后该进程停止执行，由所有小于BigStride/2的进程执行。因此最大步长-最小步长<=BigStride/最小特权级
已知以上结论，考虑溢出的情况下，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 partial_cmp 函数，假设两个 Stride 永远不会相等。
    ```rust
    use core::cmp::Ordering;

    struct Stride(u64);

    impl PartialOrd for Stride {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            let self_task_info = self.inner.exclusive_access().task_info;
            let other_task_info = other.inner.exclusive_access().task_info;
            other_task_info.stride.partial_cmp(&self_task_info.stride)
        }
    }

    impl PartialEq for Stride {
        fn eq(&self, other: &Self) -> bool {
            false
        }
    }
    ```