use honest_detection::experiment::{run_seeds};
use honest_detection::{Mode};

#[derive(Debug, Clone)]
pub struct Result {
    mode: Mode,
    violated_rate:f64,
    report_false_rate:f64,
    mean_harms:f64,
}

fn main()  {
    let modes:[Mode;3] = [Mode::None, Mode::Immediate, Mode::Retro];
    let mut results:Vec<Result> = Vec::new();
    let results: Vec<Result> = modes.iter().map(|mode| {
        let (violated_vec,report_false_vec, harms_vec) = run_seeds(mode); 
        let sample_num = violated_vec.len();
        let violated_rate = violated_vec.iter().sum::<f64>() / sample_num as f64;
        let report_false_rate = report_false_vec.iter().sum::<f64>() / sample_num as f64;
        let mean_harms = harms_vec.iter().sum::<f64>() / sample_num as f64;
        Result{mode:mode.clone(),
            violated_rate:violated_rate,
            report_false_rate:report_false_rate,
            mean_harms:mean_harms }
            })
        .collect();
    println!("results: {:?}", results);
}