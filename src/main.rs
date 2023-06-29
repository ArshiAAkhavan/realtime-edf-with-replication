use rand::seq::SliceRandom;

use scheduling::uunifast;
use scheduling::Task;
use scheduling::TaskList;

fn main() {
    let k = 2;
    let periods = vec![10, 20, 30, 40, 50, 60];
    let mut rng = rand::thread_rng();
    const NUM_TASK: usize = 10;

    let mut tasks = Vec::with_capacity(NUM_TASK);
    for (id, utilization) in uunifast(NUM_TASK, 0.9).iter().enumerate() {
        let period = periods.choose(&mut rng).unwrap();
        let wcet = ((*period as f32) * utilization) as usize;
        tasks.push(Task::new(id, wcet, *period))
    }
    let tasklist = TaskList::from(tasks).with_replication(k);
    let dispatched_list = tasklist.first_fit(5).unwrap();
    for tasklist in dispatched_list {
        let mut joblist = tasklist.jobs_till_hyperperiod();
        joblist.schedule();
        println!("{:?}", joblist.timeline(tasklist.hyperperiod()))
    }
}
