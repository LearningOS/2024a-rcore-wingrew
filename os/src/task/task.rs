//! Types related to task management

use alloc::boxed::Box;

use super::TaskContext;

use crate::config::MAX_SYSCALL_NUM;

/// taskinfo
#[derive(Copy, Clone)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
}

impl TaskInfo {
    /// 初始化 TaskInfo，提供默认值
    pub fn new() -> Self {
        TaskInfo {
            status: TaskStatus::Ready, // 根据需要设置初始状态
            syscall_times: [0; MAX_SYSCALL_NUM],
            time: 0,
        }
    }
}

/// The task control block (TCB) of a task.
#[derive(Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// The task info
    pub task_info:Box<TaskInfo>,
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
