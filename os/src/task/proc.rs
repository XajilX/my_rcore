use core::mem::size_of;

use alloc::{string::String, sync::{Arc, Weak}, vec::Vec};

use alloc::vec;

use crate::{fs::{stdio::{Stdin, Stdout}, File}, mm::{memset::{MemSet, KERN_SPACE}, pagetab::PageTab}, sync::{CondVar, Mutex, Semaphore, UThrCell, UThrRefMut}, task::pid::pid_alloc, trap::{context::TrapContext, trap_handler}};

use super::{add_task, allocator::IdAlloc, manager::reg_proc, pid::PidHandle, signal::SignalFlags, task::TaskControlBlock};

pub struct ProcControlBlock {
    pub pid: PidHandle,
    mut_part: UThrCell<PCBMut>
}

pub struct PCBMut {
    pub is_zombie: bool,
    pub memset: MemSet,
    pub parent: Option<Weak<ProcControlBlock>>,
    pub children: Vec<Arc<ProcControlBlock>>,
    pub fd_table: Vec<Option<Arc<dyn File + Sync + Send>>>,
    pub exit_code: i32,
    pub signals: SignalFlags,
    pub tasks: Vec<Option<Arc<TaskControlBlock>>>,
    pub task_alloc: IdAlloc,
    pub mutexes: Vec<Option<Arc<Mutex>>>,
    pub semaphores: Vec<Option<Arc<Semaphore>>>,
    pub condvars: Vec<Option<Arc<CondVar>>>
}

impl ProcControlBlock {
    pub fn new(elf_data: &[u8]) -> Arc<Self> {
        let (memset, ustack_base, entry_point) = MemSet::from_elf(elf_data);
        let pid = pid_alloc();
        let proc = Arc::new(Self {
            pid,
            mut_part: unsafe {
                UThrCell::new(PCBMut {
                    is_zombie:false,
                    memset,
                    parent: None,
                    children: Vec::new(),
                    fd_table: vec![
                        Some(Arc::new(Stdin)),
                        Some(Arc::new(Stdout)),
                        Some(Arc::new(Stdout)),
                    ],
                    exit_code: 0,
                    signals: SignalFlags::empty(),
                    tasks: Vec::new(),
                    task_alloc: IdAlloc::new(),
                    mutexes: Vec::new(),
                    semaphores: Vec::new(),
                    condvars: Vec::new()
                })
            }
        });
        // main thread
        let task = Arc::new(TaskControlBlock::new(
            proc.clone(),
            ustack_base,
            true
        ));
        let task_inner = task.get_mutpart();
        let trap_cx = task_inner.get_trap_cx();
        let ustack_top = task_inner.res.as_ref().unwrap().ustack_top();
        let kstack_top = task.kern_stack.get_top();
        drop(task_inner);
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            ustack_top,
            KERN_SPACE.get_refmut().get_atp_token(),
            kstack_top,
            trap_handler as usize
        );
        let mut proc_inner = proc.get_mutpart();
        proc_inner.tasks.push(Some(task.clone()));
        drop(proc_inner);
        reg_proc(proc.getpid(), &proc);
        add_task(task);
        proc
    }
    pub fn fork(self: &Arc<Self>) -> Arc<Self> {
        let mut par_mut = self.get_mutpart();
        assert_eq!(par_mut.task_count(), 1);
        let memset = par_mut.memset.clone();
        let pid = pid_alloc();
        //  copy fd table
        let new_fd_tab: Vec<Option<Arc<dyn File + Send + Sync>>> = par_mut.fd_table.iter()
            .map(|ofd| match ofd {
                    Some(file) => Some(file.clone()),
                    None => None
            }).collect();
        let pcb = Arc::new(Self {
            pid,
            mut_part: unsafe {
                UThrCell::new(PCBMut {
                    is_zombie: false,
                    memset,
                    parent: Some(Arc::downgrade(self)),
                    children: Vec::new(),
                    fd_table: new_fd_tab,
                    exit_code: 0,
                    signals: SignalFlags::empty(),
                    tasks: Vec::new(),
                    task_alloc: IdAlloc::new(),
                    mutexes: Vec::new(),
                    semaphores: Vec::new(),
                    condvars: Vec::new()
                })
            }
        });
        par_mut.children.push(pcb.clone());
        let task = Arc::new(TaskControlBlock::new(
            pcb.clone(),
            par_mut.get_task(0).get_mutpart()
                .res.as_ref().unwrap().ustack_base,
            false
        ));
        let mut child_inner = pcb.get_mutpart();
        child_inner.tasks.push(Some(task.clone()));
        drop(child_inner);

        let task_inner = task.get_mutpart();
        let trap_cx = task_inner.get_trap_cx();
        trap_cx.kern_sp = task.kern_stack.get_top();
        drop(task_inner);
        reg_proc(pcb.getpid(), &pcb);
        add_task(task);
        pcb
    }
    pub fn exec(self: &Arc<Self>, elf_data: &[u8], args: &Vec<String>) {
        assert_eq!(self.get_mutpart().task_count(), 1);
        let (memset, ustack_base, entry_pt) = MemSet::from_elf(elf_data);
        let new_token = memset.get_atp_token();
        self.get_mutpart().memset = memset;

        let task = self.get_mutpart().get_task(0);
        let mut inner = task.get_mutpart();
        inner.res.as_mut().unwrap().ustack_base = ustack_base;
        inner.res.as_mut().unwrap().alloc_user_res();
        inner.trap_cx_ppn = inner.res.as_ref().unwrap().trap_cx_ppn();

        // push args into user stack
        let mut user_sp = inner.res.as_ref().unwrap().ustack_top();
        user_sp -= (args.len() + 1) * size_of::<usize>();
        let argv_base = user_sp;
        let mut argv: Vec<_> = (0..=args.len()).map(|i| {
            PageTab::from_token(new_token).trans_mut(
                (argv_base + i * size_of::<usize>()) as *mut usize
            )
        }).collect();
        *argv[args.len()] = 0;
        for i in 0..args.len() {
            user_sp -= args[i].len() + 1;
            *argv[i] = user_sp;
            for (j, b) in args[i].as_bytes().iter().enumerate() {
                *(PageTab::from_token(new_token).trans_mut(
                    (user_sp + j) as *mut u8
                )) = *b;
            }
            *(PageTab::from_token(new_token).trans_mut(
                (user_sp + args[i].len()) as *mut u8
            )) = 0;
        }
        // align
        user_sp -= user_sp % size_of::<usize>();

        let mut trap_cx = TrapContext::app_init_context(
            entry_pt,
            user_sp,
            KERN_SPACE.get_refmut().get_atp_token(),
            task.kern_stack.get_top(),
            trap_handler as usize
        );

        trap_cx.reg[11] = argv_base;
        *inner.get_trap_cx() = trap_cx;
    }
    pub fn getpid(&self) -> usize {
        self.pid.0
    }
    pub fn get_mutpart(&self) -> UThrRefMut<'_, PCBMut> {
        self.mut_part.get_refmut()
    }
}

impl PCBMut {
    pub fn get_atp_token(&self) -> usize {
        self.memset.get_atp_token()
    }
    pub fn alloc_tid(&mut self) -> usize {
        self.task_alloc.alloc()
    }
    pub fn dealloc_tid(&mut self, tid: usize) {
        self.task_alloc.dealloc(tid)
    }
    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }
    pub fn get_task(&self, tid: usize) -> Arc<TaskControlBlock> {
        self.tasks[tid].as_ref().unwrap().clone()
    }

    pub fn alloc_new_id<T>(table: &mut Vec<Option<T>>) -> usize {
        if let Some(id) = (0..table.len()).find(|id| {
            table[*id].is_none()
        }) {
            id
        } else {
            table.push(None);
            table.len() - 1
        }
    }
}