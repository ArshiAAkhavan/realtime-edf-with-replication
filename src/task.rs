use std::collections::BinaryHeap;
use std::collections::HashSet;
use std::ops::Neg;

use crate::job::Job;
use crate::job::JobList;

#[derive(Clone, Debug)]
pub struct Task {
    id: usize,
    wcet: usize,
    period: usize,
}

impl Task {
    pub fn new(id: usize, wcet: usize, period: usize) -> Self {
        Self { id, wcet, period }
    }

    pub(crate) fn jobs_till(&self, deadline: usize) -> JobList {
        let mut now = 0_usize;
        let mut iteration = 0;
        let mut jobs = JobList::new();
        while now < deadline {
            let deadline = now + self.period;
            jobs.push(Job::new(self.id, iteration, now, self.wcet, deadline));
            iteration += 1;
            now += self.period;
        }
        jobs
    }
    pub fn utilization(&self) -> f32 {
        self.wcet as f32 / self.period as f32
    }
}

enum ProcessorError {
    TaskAlreadyExists(Task),
    NotEnoughCapacity(Task),
}
struct Processor {
    tasks: Vec<Task>,
    capacity: f32,
    task_ids: HashSet<usize>,
}

impl Processor {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
            capacity: 1.0,
            task_ids: HashSet::new(),
        }
    }
    fn push(&mut self, task: Task) -> Result<(), ProcessorError> {
        if self.task_ids.contains(&task.id) {
            Err(ProcessorError::TaskAlreadyExists(task))
        } else if self.capacity < task.utilization() {
            Err(ProcessorError::NotEnoughCapacity(task))
        } else {
            self.capacity -= task.utilization();
            self.task_ids.insert(task.id);
            self.tasks.push(task);
            Ok(())
        }
    }
    fn take(self) -> TaskList {
        TaskList {
            tasks: self.tasks,
            replication: 0,
        }
    }
}

#[derive(Debug)]
pub struct TaskList {
    tasks: Vec<Task>,
    replication: usize,
}

impl TaskList {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            replication: 0,
        }
    }
    pub fn with_replication(self, replication: usize) -> Self {
        Self {
            tasks: self.tasks,
            replication,
        }
    }
    pub fn hyperperiod(&self) -> usize {
        self.tasks
            .iter()
            .map(|t| t.period)
            .reduce(num::integer::lcm)
            .unwrap()
    }

    pub fn jobs_till_hyperperiod(&self) -> JobList {
        let hyperperiod = self.hyperperiod();

        let mut joblist = JobList::new();
        for task in &self.tasks {
            joblist.join(task.jobs_till(hyperperiod));
        }
        joblist
    }

    pub fn push(&mut self, task: Task) {
        self.tasks.push(task)
    }

    pub fn first_fit(&self, num_proc: usize) -> Result<Vec<TaskList>, Vec<TaskList>> {
        let mut processors = Vec::with_capacity(num_proc);

        for _ in 0..num_proc {
            processors.push(Processor::new());
        }

        for task in &self.tasks {
            for _ in 0..self.replication + 1 {
                let mut task = task.clone();
                let mut pushed = false;
                for proc in processors.iter_mut() {
                    match proc.push(task) {
                        Ok(_) => {
                            pushed = true;
                            break;
                        }
                        Err(ProcessorError::NotEnoughCapacity(t)) => task = t,
                        Err(ProcessorError::TaskAlreadyExists(t)) => task = t,
                    }
                }
                if !pushed {
                    return Err(processors.into_iter().map(|p| p.take()).collect());
                }
            }
        }
        Ok(processors.into_iter().map(|p| p.take()).collect())
    }

    pub fn worst_fit(&self, num_proc: usize) -> Result<Vec<TaskList>, Vec<TaskList>> {
        struct ProcWrapper(Processor);
        impl PartialEq for ProcWrapper {
            fn eq(&self, other: &Self) -> bool {
                self.0.capacity == other.0.capacity
            }
        }
        impl Eq for ProcWrapper {
            fn assert_receiver_is_total_eq(&self) {}
        }
        impl Ord for ProcWrapper {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.0.capacity.neg().total_cmp(&other.0.capacity.neg())
            }
        }
        impl PartialOrd for ProcWrapper {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.0.capacity.neg().partial_cmp(&other.0.capacity.neg())
            }
        }

        let mut processors = BinaryHeap::with_capacity(num_proc);
        for _ in 0..num_proc {
            processors.push(ProcWrapper(Processor::new()));
        }

        for task in &self.tasks {
            for _ in 0..self.replication + 1 {
                let mut task = task.clone();
                let mut invalid_processors = Vec::new();
                while let Some(mut p) = processors.pop() {
                    match p.0.push(task) {
                        Ok(_) => {
                            processors.extend(invalid_processors);
                            processors.push(p);
                            break;
                        }
                        Err(ProcessorError::TaskAlreadyExists(t)) => {
                            invalid_processors.push(p);
                            task = t;
                        }
                        Err(ProcessorError::NotEnoughCapacity(t)) => {
                            invalid_processors.push(p);
                            task = t;
                        }
                    }
                }
                if processors.is_empty() {
                    return Err(processors
                        .into_vec()
                        .into_iter()
                        .map(|w| w.0.take())
                        .collect());
                }
            }
        }
        Ok(processors
            .into_vec()
            .into_iter()
            .map(|w| w.0.take())
            .collect())
    }

    pub fn best_fit(&self, num_proc: usize) -> Result<Vec<TaskList>, Vec<TaskList>> {
        struct ProcWrapper(Processor);
        impl PartialEq for ProcWrapper {
            fn eq(&self, other: &Self) -> bool {
                self.0.capacity == other.0.capacity
            }
        }
        impl Eq for ProcWrapper {
            fn assert_receiver_is_total_eq(&self) {}
        }
        impl Ord for ProcWrapper {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.0.capacity.total_cmp(&other.0.capacity)
            }
        }
        impl PartialOrd for ProcWrapper {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.0.capacity.partial_cmp(&other.0.capacity)
            }
        }

        let mut processors = BinaryHeap::with_capacity(num_proc);
        for _ in 0..num_proc {
            processors.push(ProcWrapper(Processor::new()));
        }

        for task in &self.tasks {
            for _ in 0..self.replication + 1 {
                let mut task = task.clone();
                let mut invalid_processors = Vec::new();
                let mut pushed = false;
                while let Some(mut p) = processors.pop() {
                    match p.0.push(task) {
                        Ok(_) => {
                            processors.push(p);
                            processors.extend(invalid_processors);
                            pushed = true;
                            break;
                        }
                        Err(ProcessorError::TaskAlreadyExists(t)) => {
                            invalid_processors.push(p);
                            task = t;
                        }
                        Err(ProcessorError::NotEnoughCapacity(_)) => {
                            processors.push(p);
                            processors.extend(invalid_processors);
                            break;
                        }
                    }
                }
                if !pushed {
                    return Err(processors
                        .into_vec()
                        .into_iter()
                        .map(|w| w.0.take())
                        .collect());
                }
            }
        }
        Ok(processors
            .into_vec()
            .into_iter()
            .map(|w| w.0.take())
            .collect())
    }
}

impl From<Vec<Task>> for TaskList {
    fn from(tasks: Vec<Task>) -> Self {
        Self {
            tasks,
            replication: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_tasks() {
        let t1 = Task::new(1, 2, 6);
        let t2 = Task::new(2, 2, 8);
        let t3 = Task::new(3, 3, 12);
        let tasklist = TaskList::from(vec![t1, t2, t3]).with_replication(1);
        for tasks in tasklist.first_fit(2).unwrap() {
            assert_eq!(
                tasks.tasks.iter().map(|t| t.id).collect::<Vec<usize>>(),
                vec![1, 2, 3]
            );
        }

        for tasks in tasklist.worst_fit(2).unwrap() {
            assert_eq!(
                tasks.tasks.iter().map(|t| t.id).collect::<Vec<usize>>(),
                vec![1, 2, 3]
            );
        }

        for tasks in tasklist.best_fit(2).unwrap() {
            assert_eq!(
                tasks.tasks.iter().map(|t| t.id).collect::<Vec<usize>>(),
                vec![1, 2, 3]
            );
        }
    }

    #[test]
    fn num_proc_lt_replication() {
        let t1 = Task::new(1, 2, 6);
        let t2 = Task::new(2, 2, 8);
        let t3 = Task::new(3, 3, 12);
        let tasklist = TaskList::from(vec![t1, t2, t3]).with_replication(1);
        assert!(matches!(tasklist.first_fit(1), Err(_)));
        assert!(matches!(tasklist.worst_fit(1), Err(_)));
        assert!(matches!(tasklist.best_fit(1), Err(_)));
    }

    #[test]
    fn total_util_gt_num_proc() {
        let t1 = Task::new(1, 8, 10);
        let t2 = Task::new(2, 6, 10);
        let t3 = Task::new(3, 2, 10);
        let tasklist = TaskList::from(vec![t1, t2, t3]).with_replication(4);
        assert!(matches!(tasklist.first_fit(7), Err(_)));
        assert!(matches!(tasklist.worst_fit(7), Err(_)));
        assert!(matches!(tasklist.best_fit(7), Err(_)));
    }

    #[test]
    fn total_util_le_num_proc() {
        let t1 = Task::new(1, 4, 10);
        let t2 = Task::new(2, 3, 10);
        let t3 = Task::new(3, 1, 10);
        let tasklist = TaskList::from(vec![t1, t2, t3]).with_replication(4);
        assert!(matches!(tasklist.first_fit(5), Ok(_)));
        assert!(matches!(tasklist.worst_fit(5), Ok(_)));
        assert!(matches!(tasklist.best_fit(5), Ok(_)));
    }
    #[test]
    fn first_fit() {
        let t1 = Task::new(1, 7, 10);
        let t2 = Task::new(2, 1, 10);
        let t3 = Task::new(3, 1, 10);
        let tasklist = TaskList::from(vec![t1, t2, t3]).with_replication(1);
        let ids: Vec<Vec<usize>> = tasklist
            .first_fit(3)
            .unwrap()
            .iter()
            .map(|tasks| tasks.tasks.iter().map(|t| t.id).collect::<Vec<usize>>())
            .collect();
        assert_eq!(ids[0], vec![1, 2, 3]);
        assert_eq!(ids[1], vec![1, 2, 3]);
        assert_eq!(ids[2], Vec::new());
    }

    #[test]
    fn best_fit() {
        let t1 = Task::new(1, 7, 10);
        let t2 = Task::new(2, 1, 10);
        let t3 = Task::new(3, 1, 10);
        let tasklist = TaskList::from(vec![t1, t2, t3]).with_replication(1);
        let ids: Vec<Vec<usize>> = tasklist
            .best_fit(3)
            .unwrap()
            .iter()
            .map(|tasks| tasks.tasks.iter().map(|t| t.id).collect::<Vec<usize>>())
            .collect();
        assert_eq!(ids[0], vec![2, 3]);
        assert_eq!(ids[1], vec![1, 3]);
        assert_eq!(ids[2], vec![1, 2]);
    }

    #[test]
    fn worst_fit() {
        let t1 = Task::new(1, 4, 10);
        let t2 = Task::new(2, 4, 10);
        let t3 = Task::new(3, 4, 10);
        let t4 = Task::new(4, 4, 10);
        let tasklist = TaskList::from(vec![t1, t2, t3, t4]).with_replication(1);
        let ids: Vec<Vec<usize>> = tasklist
            .worst_fit(4)
            .unwrap()
            .iter()
            .map(|tasks| tasks.tasks.iter().map(|t| t.id).collect::<Vec<usize>>())
            .collect();
        assert_eq!(ids[0], vec![1, 2]);
        assert_eq!(ids[1], vec![1, 2]);
        assert_eq!(ids[2], vec![3, 4]);
        assert_eq!(ids[3], vec![3, 4]);
    }
}
