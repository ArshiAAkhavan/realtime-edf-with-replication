use rand::{self, Rng};

/// returns a vector of utilizations for `num_tasks` tasks holding:
/// sum(utilizations) < total_utilizatoin
pub fn uunifast(num_tasks: usize, total_utilization: f32) -> Vec<f32> {
    let mut rng = rand::thread_rng();

    let mut utilizations = Vec::with_capacity(num_tasks);
    let mut sum_u = total_utilization;
    for i in 1..num_tasks {
        let next_sum_u = sum_u * rng.gen::<f32>().powf(1_f32 / (num_tasks - i) as f32);
        utilizations.push(sum_u - next_sum_u);
        sum_u = next_sum_u;
    }

    const F32_JITTER: f32 = 1e-7;
    utilizations.push(sum_u - F32_JITTER);
    utilizations
}

#[cfg(test)]
mod tests {
    use rand::Rng;

    use super::*;

    #[test]
    fn sum_less_than_total_u() {
        const TEST_SIZE: i32 = 10;
        const TASK_SIZE: usize = 10;
        let mut rng = rand::thread_rng();
        for _ in 0..TEST_SIZE {
            let u = rng.gen();

            assert!(u >= uunifast(TASK_SIZE, u).iter().sum())
        }
    }
}
