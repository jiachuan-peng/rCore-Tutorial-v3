若想对进程进行管理，实现创建、退出等操作，核心就在于 fork/exec/waitpid 三个系统调用。

运行在应用态的应用软件，它们均放置在 user 目录下。在新增系统调用的时候，需要在 user/src/lib.rs 中新增一个 sys_* 的函数，它的作用是将对应的系统调用按照与内核约定的 ABI 在 syscall 中转化为一条用于触发系统调用的 ecall 的指令；还需要在用户库 user_lib 将 sys_* 进一步封装成一个应用可以直接调用的与系统调用同名的函数。三个进程模型中核心的系统调用 fork/exec/waitpid ，一个查看进程 PID 的系统调用 getpid ，还有一个允许应用程序获取用户键盘输入的 read 系统调用。

基于进程模型，我们在 user/src/bin 目录下重新实现了一组应用程序。其中有两个特殊的应用程序：用户初始程序 initproc.rs 和 shell 程序 user_shell.rs ，可以认为它们位于内核和其他应用程序之间的中间层提供一些基础功能，但是它们仍处于用户态的应用层。前者会被内核唯一自动加载、也是最早加载并执行，后者则负责从键盘接收用户输入的应用名并执行对应的应用。剩下的应用从不同层面测试了我们内核实现的正确性， usertests 可以按照顺序执行绝大部分应用，会在测试操作系统功能和正确性上为我们提供很多方便。


为了支持基于应用名而不是应用 ID 来查找应用 ELF 可执行文件，从而实现灵活的应用加载，在 os/build.rs 以及 os/src/loader.rs 中更新了 link_app.S 的格式使得它包含每个应用的名字，另外提供 get_app_data_by_name 接口获取应用的 ELF 数据。

 os/src/task/processor.rs 中的处理器管理结构 Processor 中，它负责管理 CPU 上执行的任务和一些其他信息；而 os/src/task/manager.rs 中的任务管理器 TaskManager 仅负责管理所有任务。


进程的 PID 将作为查找进程控制块的索引，这样就可以通过进程的 PID 来查找到进程的内核栈等各种进程相关信息。 同时还面向进程控制块提供相应的资源自动回收机制。具体实现可以参考 os/src/task/pid.rs 。

有了这些数据结构的支撑，我们在本章第三小节 进程管理机制的设计实现 实现进程管理机制。它可以分成如下几个方面：

初始进程的创建：在内核初始化的时候需要调用 os/src/task/mod.rs 中的 add_initproc 函数，它会调用 TaskControlBlock::new 读取并解析初始应用 initproc 的 ELF 文件数据并创建初始进程 INITPROC ，随后会将它加入到全局任务管理器 TASK_MANAGER 中参与调度。

进程切换机制：当一个进程退出或者是主动/被动交出 CPU 使用权之后，需要由内核将 CPU 使用权交给其他进程。我们沿用 os/src/task/mod.rs 中的 suspend_current_and_run_next 和 exit_current_and_run_next 两个接口来实现进程切换功能.

进程调度机制：在进程切换的时候我们需要选取一个进程切换过去。选取进程逻辑可以参考 os/src/task/manager.rs 中的 TaskManager::fetch_task 方法。

进程生成机制：这主要是指 fork/exec 两个系统调用。它们的实现分别可以在 os/src/syscall/process.rs 中找到，分别基于 os/src/process/task.rs 中的 TaskControlBlock::fork/exec 。

进程资源回收机制：当一个进程主动退出或出错退出的时候，在 exit_current_and_run_next 中会立即回收一部分资源并在进程控制块中保存退出码；而需要等到它的父进程通过 waitpid 系统调用（与 fork/exec 两个系统调用放在相同位置）捕获到它的退出码之后，它的进程控制块才会被回收，从而该进程的所有资源都被回收。

进程的 I/O 输入机制：支持用户终端 user_shell 读取用户键盘输入的功能，它可以在 os/src/syscall/fs.rs 中找到。