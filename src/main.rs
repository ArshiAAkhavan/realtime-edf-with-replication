use clap::{Parser, ValueEnum};
use rand::seq::SliceRandom;

use scheduling::uunifast;
use scheduling::Task;
use scheduling::TaskList;

#[derive(ValueEnum, Debug, Clone)]
#[clap(rename_all = "kebab_case")]
enum DispatchAlgorithm {
    FirstFit,
    BestFit,
    WorstFit,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[clap(rename_all = "kebab_case")]
struct Cli {
    /// which dispatch algorithm to choose
    /// options are: FirstFit, WorstFit, and BestFit
    #[arg(short, long, value_enum,default_value_t=DispatchAlgorithm::FirstFit)]
    dispatch_algorithm: DispatchAlgorithm,

    /// number of tasks
    #[arg(short, long)]
    num_tasks: usize,

    /// number of CPU processors
    #[arg(short = 'c', long)]
    num_cpu: usize,

    /// replication factor.
    /// replicaton factor of 0 means there is only 1 instance of each task
    #[arg(short, long)]
    replication_factor: usize,
}

fn main() {
    let periods = vec![10, 20, 30, 40, 50, 60];
    let mut rng = rand::thread_rng();

    let cli = Cli::parse();

    let mut tasks = Vec::with_capacity(cli.num_tasks);
    for (id, utilization) in uunifast(cli.num_tasks, 0.9).iter().enumerate() {
        let period = periods.choose(&mut rng).unwrap();
        let wcet = ((*period as f32) * utilization) as usize;
        tasks.push(Task::new(id, wcet, *period))
    }
    let tasklist = TaskList::from(tasks).with_replication(cli.replication_factor);
    let dispatched_list = match cli.dispatch_algorithm {
        DispatchAlgorithm::FirstFit => tasklist.first_fit(cli.num_cpu),
        DispatchAlgorithm::BestFit => tasklist.best_fit(cli.num_cpu),
        DispatchAlgorithm::WorstFit => tasklist.worst_fit(cli.num_cpu),
    }
    .unwrap();
    for tasklist in dispatched_list {
        let mut joblist = tasklist.jobs_till_hyperperiod();
        joblist.schedule();
        println!("{:?}", joblist.timeline(tasklist.hyperperiod()))
    }
}
