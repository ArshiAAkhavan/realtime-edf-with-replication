pub struct Job {
    id: usize,
    iteration: usize,
    arrival_time: usize,
    deadline: usize,
    remaining: usize,
    log: Vec<(usize, usize)>,
    status: JobStatus,
}

enum JobStatus {
    Ready,
    Running,
    DeadlineExceeded,
    Done,
}

impl Job {
    pub fn new(id: usize, iteration: usize, arrival_time: usize, wcet: usize, deadline: usize) -> Self {
        Self {
            id,
            iteration,
            arrival_time,
            deadline,
            remaining: wcet,
            log: Vec::new(),
            status: JobStatus::Ready,
        }
    }
    fn run(&mut self, from: usize, to: usize) -> usize {
        let untill = *[to, self.deadline, from + self.remaining]
            .iter()
            .min()
            .unwrap();
        let duration = untill - from;
        self.remaining -= duration;
        self.log.push((from, untill));

        if untill + self.remaining > self.deadline {
            self.status = JobStatus::DeadlineExceeded;
        } else if self.remaining == 0 {
            self.status = JobStatus::Done;
        } else {
            self.status = JobStatus::Running;
        }
        untill - from
    }
}

pub struct JobList {
    jobs: Vec<Job>,
}

impl JobList {
    pub fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    pub fn push(&mut self, job: Job) {
        self.jobs.push(job);
    }
    
    pub fn pop(&mut self) -> Option<Job> {
        self.jobs.pop()
    }

    pub fn join(&mut self, other: Self) -> &mut Self {
        self.jobs.extend(other.jobs);
        self
    }

    pub fn schedule(&mut self) {
        // arrival time decending
        self.jobs.sort_by_key(|x| x.arrival_time);
        self.jobs.reverse();

        let mut finished_jobs = Vec::new();
        let mut ready_jobs: Vec<Job> = Vec::new();
        let mut now = 0;
        while let Some(new_job) = self.jobs.pop() {
            while now < new_job.arrival_time {
                // sort by deadline decending
                ready_jobs.sort_by_key(|x| x.deadline);
                ready_jobs.reverse();

                // ready_jobs can run in this slack time
                if let Some(mut active_job) = ready_jobs.pop() {
                    let duration = active_job.run(now, new_job.arrival_time);
                    now += duration;
                    match active_job.status {
                        JobStatus::Ready | JobStatus::Running => ready_jobs.push(active_job),
                        JobStatus::DeadlineExceeded | JobStatus::Done => {
                            finished_jobs.push(active_job)
                        }
                    }
                } else {
                    break;
                }
            }
            now = new_job.arrival_time;
            ready_jobs.push(new_job);
        }
        self.jobs = finished_jobs;
    }
    
    pub fn timeline(&self, to: usize) -> Vec<(usize, usize)> {
        let mut timeline = vec![(0, 0); to];
        for job in self.jobs.iter() {
            for run in &job.log {
                for cycle in run.0..run.1 {
                    timeline[cycle] = (job.id, job.iteration)
                }
            }
        }
        timeline
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Task;

    #[test]
    fn smoke() {
        let t1 = Task::new(1, 2, 6);
        let t2 = Task::new(2, 2, 8);
        let t3 = Task::new(3, 3, 12);
        let mut jobs = JobList::new();
        jobs.join(t1.jobs_till(24))
            .join(t2.jobs_till(24))
            .join(t3.jobs_till(24));
        let timeline = vec![
            (1, 0), (1, 0), (2, 0), (2, 0), (3, 0), (3, 0),
            (3, 0), (1, 1), (1, 1), (2, 1), (2, 1), (0, 0),
            (1, 2), (1, 2), (3, 1), (3, 1), (3, 1), (0, 0),
            (0, 0), (0, 0), (0, 0), (0, 0), (0, 0), (0, 0),
        ];

        jobs.schedule();
        assert_eq!(timeline, jobs.timeline(24));
    }

    #[test]
    fn smoke2() {
        let t1 = Task::new(1, 1, 3);
        let t2 = Task::new(2, 1, 4);
        let t3 = Task::new(3, 2, 8);
        let mut jobs = JobList::new();
        jobs.join(t1.jobs_till(24))
            .join(t2.jobs_till(24))
            .join(t3.jobs_till(24));
        let timeline = vec![
            (1, 0), (2, 0), (3, 0), (1, 1), (3, 0), (2, 1),
            (1, 2), (0, 0), (2, 2), (1, 3), (3, 1), (3, 1),
            (1, 4), (2, 3), (0, 0), (1, 5), (2, 4), (3, 2),
            (1, 6), (3, 2), (2, 5), (0, 0), (0, 0), (0, 0),
        ];

        jobs.schedule();
        assert_eq!(timeline, jobs.timeline(24));
    }
}
