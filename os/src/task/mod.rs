mod context;
mod switch;
mod task;

use core::primitive;

use crate::config::{MAX_APP_NUM, BIG_STRIDE};
use crate::console::print;
use crate::loader::{get_num_app, init_app_cx};
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use crate::sync::UPSafeCell;

pub use context::TaskContext;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

pub struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [
            TaskControlBlock {
                task_cx: TaskContext::zero_init(),
                task_status: TaskStatus::UnInit,
                task_priority:16,
                task_stride:0
            };
            MAX_APP_NUM
        ];
        for i in 0..num_app {
            tasks[i].task_cx = TaskContext::goto_restore(init_app_cx(i));
            tasks[i].task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe { UPSafeCell::new(TaskManagerInner {
                tasks,
                current_task: 0,
            })},
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        self.add_stride();
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(
                &mut _unused as *mut TaskContext,
                next_task_cx_ptr,
            );
        }
        panic!("unreachable in run_first_task!");
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    /*fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| {
                inner.tasks[*id].task_status == TaskStatus::Ready
            })
    }*/
    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let num_app=TASK_MANAGER.num_app;
        let mut maxx=usize::MAX;
        let mut next=10000;
        for i in (0..num_app){
            if(inner.tasks[i].task_stride<maxx&&inner.tasks[i].task_status==TaskStatus::Ready){
                maxx=inner.tasks[i].task_stride;
                next=i;
            }
        }
        if(next==10000){return None;}
        else {return Some(next);}
    }
    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
           
            
            let mut inner = self.inner.exclusive_access();
            /*for i in (0..TASK_MANAGER.num_app){
                unsafe{print!("app{} stride {} ",i,inner.tasks[i].task_stride)};
            }
            unsafe{print!("\n")};*/
            unsafe{println!("run app{}",next)};
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;

            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            self.add_stride();
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(
                    current_task_cx_ptr,
                    next_task_cx_ptr,
                );
            }
            // go back to user mode
        } else {
            panic!("All applications completed!");
        }
    }
    fn get_current_app(&self) -> usize {
        let inner = self.inner.exclusive_access();
        return inner.current_task ;

    }
    fn set_priority(&self,priority:isize) -> isize {
        //unsafe{println!("set_priority")};
        let mut inner = self.inner.exclusive_access();
        let current=inner.current_task;
        inner.tasks[current].task_priority=priority as usize;
        return priority;
    }
    fn add_stride(&self)
    {
        let mut inner = self.inner.exclusive_access();
        let current=inner.current_task;
        let priority=inner.tasks[current].task_priority;
        inner.tasks[current].task_stride+=(BIG_STRIDE/priority) as usize;

    }
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}
pub fn get_current_app() -> usize{
    return TASK_MANAGER.get_current_app();
}
pub fn set_priority(priority:isize) -> isize{
    TASK_MANAGER.set_priority(priority)
}