use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use rand::seq::SliceRandom;

use scheduling::uunifast;
use scheduling::Task;
use scheduling::TaskList;
use serde_json::json;

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

    /// total utilization of the generated tasks.
    #[arg(short, long)]
    utilization: f32,

    /// path to output file
    #[arg(short, long)]
    output_path: PathBuf,
}

fn main() -> std::io::Result<()> {
    let periods = vec![100, 200, 300, 400, 500, 600];
    let mut rng = rand::thread_rng();

    let cli = Cli::parse();

    let mut tasks = Vec::with_capacity(cli.num_tasks);
    for (id, utilization) in uunifast(cli.num_tasks, cli.utilization).iter().enumerate() {
        let period = periods.choose(&mut rng).unwrap();
        let wcet = ((*period as f32) * utilization) as usize;
        tasks.push(Task::new(id, wcet, *period))
    }
    let tasklist = TaskList::from(tasks).with_replication(cli.replication_factor);
    let dispatched_list = match cli.dispatch_algorithm {
        DispatchAlgorithm::FirstFit => tasklist.first_fit(cli.num_cpu),
        DispatchAlgorithm::BestFit => tasklist.best_fit(cli.num_cpu),
        DispatchAlgorithm::WorstFit => tasklist.worst_fit(cli.num_cpu),
    };
    let dispatched_list = match dispatched_list {
        Ok(tasks) => tasks,
        Err(_) => panic!("couldn't dispatch jobs into CPUs"),
    };
    let mut reports = Vec::new();
    for (i, tasklist) in dispatched_list.iter().enumerate() {
        let mut joblist = tasklist.jobs_till_hyperperiod();
        joblist.schedule();
        reports.push(json!({
            "cpu": i,
            "report": joblist.report(tasklist.hyperperiod()),
        }));
    }
    let json_string = serde_json::to_string_pretty(&reports).unwrap();
    std::fs::write(cli.output_path, json_string)
}
