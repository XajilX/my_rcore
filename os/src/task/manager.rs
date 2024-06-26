use alloc::{collections::{BTreeMap, VecDeque}, sync::Arc};

use lazy_static::lazy_static;

use crate::sync::UThrCell;

use super::{proc::ProcControlBlock, task::TaskControlBlock};


struct TaskMan {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}
impl TaskMan {
    pub fn new() -> Self {
        Self { ready_queue: VecDeque::new() }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
    pub fn remove(&mut self, task: Arc<TaskControlBlock>) {
        if let Some((id,_)) = self.ready_queue.iter().enumerate()
            .find(|(_, t)| Arc::as_ptr(t) == Arc::as_ptr(&task))
        {
            self.ready_queue.remove(id);
        }
    }
}
struct PidTab {
    table: BTreeMap<usize, Arc<ProcControlBlock>>
}
impl PidTab {
    pub fn new() -> Self {
        Self { table: BTreeMap::new() }
    }
    pub fn reg_proc(&mut self, pid: usize, proc: &Arc<ProcControlBlock>) {
        self.table.insert(pid, proc.clone());
    }
    pub fn get_proc(&self, pid: usize) -> Option<Arc<ProcControlBlock>> {
        self.table.get(&pid).map(Arc::clone)
    }
    pub fn unreg_proc(&mut self, pid: usize) {
        assert!(self.table.remove(&pid).is_some());
    }
}

lazy_static! {
    static ref TASKMAN: UThrCell<TaskMan> = unsafe {
        UThrCell::new(TaskMan::new())
    };
    static ref PIDTAB: UThrCell<PidTab> = unsafe {
        UThrCell::new(PidTab::new())
    };
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASKMAN.get_refmut().add(task);
}
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASKMAN.get_refmut().fetch()
}
pub fn remove_task(task: Arc<TaskControlBlock>) {
    TASKMAN.get_refmut().remove(task)
}

pub fn reg_proc(pid: usize, proc: &Arc<ProcControlBlock>) {
    PIDTAB.get_refmut().reg_proc(pid, proc);
}
pub fn get_proc(pid: usize) -> Option<Arc<ProcControlBlock>> {
    PIDTAB.get_refmut().get_proc(pid)
}
pub fn unreg_proc(pid: usize) {
    PIDTAB.get_refmut().unreg_proc(pid);
}
