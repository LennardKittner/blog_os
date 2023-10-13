use core::task::{Waker, Context, Poll};

use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use crossbeam_queue::ArrayQueue;
use x86_64::instructions::interrupts::{self, enable_and_hlt};

use super::{TaskID, Task};

const TASK_CAPACITY: usize = 100;

struct TaskWaker {
    task_id: TaskID,
    task_queue: Arc<ArrayQueue<TaskID>>
}

impl TaskWaker {
    fn new(task_id: TaskID, task_queue: Arc<ArrayQueue<TaskID>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue
        }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("Task queue full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

pub struct Executor {
    tasks: BTreeMap<TaskID, Task>,
    task_queue: Arc<ArrayQueue<TaskID>>,
    waker_cache: BTreeMap<TaskID, Waker>
}

impl Executor {
    pub fn new() -> Self {
        Executor { 
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(TASK_CAPACITY)),
            waker_cache: BTreeMap::new()
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task_id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("Task queue full");
    }

    fn run_ready_tasks(&mut self) {
        // destructure `self` to avoid borrow checker errors
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue // task no longer exists
            };
            let waker = waker_cache.entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remove it and its cached waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => ()
            }
        }
    }

    //TODO: allow task spawning after run through shared queue
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        interrupts::disable();
        if self.task_queue.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}