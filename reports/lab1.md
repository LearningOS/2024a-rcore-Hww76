
# 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
    无
2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
    无
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
# 实现的功能
修改任务控制块，将TaskInfo有关的数据保存在TCB中，包括系统调用次数数组syscall_times、first_run和first_run_time，first_run初始值为false
TaskInfo需要的数据获取方式：
1. 任务运行时间=当前时间-第一次运行时间。在执行run_next_task进行任务切换时，获取下一个任务的TCB，判断first_run是否为true，只有当任务第一次运行时将first_run设置为true，并将此时的时间设置为任务第一次运行的时间。‘
2. 系统调用次数：在执行syscall时会传入syscall_id，此时更新当前TCB的syscall_times数组的第syscall_id元素。

执行sys_task_info系统调用时，将TCB内的数据写入函数参数即可。
# 简答作业
1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 三个 bad 测例 (ch2b_bad_*.rs) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。
    ```
    [kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
    [kernel] IllegalInstruction in application, kernel killed it.
    [kernel] IllegalInstruction in application, kernel killed it.
    ```
1. 深入理解 trap.S 中两个函数 __alltraps 和 __restore 的作用，并回答如下问题:
    1. L40：刚进入 __restore 时，a0 代表了什么值。请指出 __restore 的两种使用情景。
        a0代表内核栈的栈基址
        运行第一个app时，从内核态回到用户态都需要通过__restore
    2. L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。
        ```
        ld t0, 32*8(sp)
        ld t1, 33*8(sp)
        ld t2, 2*8(sp)
        csrw sstatus, t0
        csrw sepc, t1
        csrw sscratch, t2
        ```
        处理了sstatus和sepc寄存器
        sstatus保存了当前特权级属性，sstatus的spp字段记录当前是用户态还是内核态。
        sepc保存了app的首地址，执行sret时，根据sepc内的值设置pc，让处理器跳转到app首地址执行
    3. L50-L56：为何跳过了 x2 和 x4？
        ```
        ld x1, 1*8(sp)
        ld x3, 3*8(sp)
        .set n, 5
        .rept 27
            LOAD_GP %n
            .set n, n+1
        .endr
        ```
        x2保存在sscratch中，在于sp进行csrrw时恢复到sp中。
        tp(x4) 寄存器，除非我们手动出于一些特殊用途使用它，否则一般也不会被用到，所以也不用恢复。
    4. L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？
        ``` 
        csrrw sp, sscratch, sp
        ```
        指令执行之后，sp保存的是用户态的栈基址，sscratch保存的是内核态的基址。
    5. __restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？
        状态切换在L61的sret指令，该指令设置pc值为sepc，特权级为sstatus的spp字段，L46将用户态程序的sstatus从t2恢复到了sstatus寄存器中。
    6. L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？
        ```
        csrrw sp, sscratch, sp
        ```
        指令执行之后，sp保存的是内核态的栈基址，sscratch保存的是用户态的基址。
    7. 从 U 态进入 S 态是哪一条指令发生的？
        syscall中的eccall指令
