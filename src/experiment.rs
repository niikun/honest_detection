use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

use crate::{Agent, Agents};
use crate::{Params, Mode, ReportStates, ReportState, Actions, Action, round, get_harms};

pub fn run(mode:&Mode, seed:u64) ->(Agents, Vec<Actions>, Vec<ReportStates>){
    let mut params = Params::default();
    params.mode = mode.clone();
    params.seed = seed;
    let mut rng = StdRng::seed_from_u64(seed);
    let mut agents_vec:Vec<Agent> = Vec::new();
    for i in 0..params.n{
        agents_vec.push(Agent::new(i, &mut rng));
    }
    let mut agents = Agents{agents:agents_vec};
    let mut all_actions:Vec<Actions> = Vec::new();
    let mut all_report_states:Vec<ReportStates> = Vec::new();
    for i in 0..params.epochs {
        let mut epsilon = params.epsilon_start - (params.epsilon_start - params.epsilon_end)/ (params.epochs as f64) *i as f64;
        let (actions, report_states) = round(&mut agents, &mut params, &mut rng, epsilon);
        if i >= ((params.epochs as f64) * 0.8) as usize{
            all_actions.push(actions);
            all_report_states.push(report_states);
        }
    }
    (agents, all_actions, all_report_states)
}

pub fn run_seeds(mode:Mode)->(Vec<f64>, Vec<f64>, Vec<f64>){
    let mut params = Params::default();
    let seeds:Vec<i32> = Vec::new();
    let mut all_mean_violated = Vec::new();
    let mut all_mean_report_false = Vec::new();
    let mut all_mean_harms = Vec::new();
    for i in 0..20 {
        let mut count_violated = 0;
        let mut count_report_false = 0;
        let mut total_harms:f64 = 0.0;
        let (agents, all_actions, all_report) =  run(&mode,i);
        all_actions.iter().for_each(|actions| {
            for action in actions.actions.iter(){
                if action == &Action::Violated{
                    count_violated += 1;
                }
            }
            let harms:Vec<f64> = get_harms(&params, actions);
            total_harms += harms.iter().sum::<f64>();
        });
        all_report.iter().for_each(|report_states| report_states.states.iter()
                                    .for_each(|state| {
                                        if state == &ReportState::ReportFalse {
                                            count_report_false += 1;
                                        }
                                    }));
        assert_eq!(all_actions.len(), all_report.len());
        let record_num = all_report.len() as f64;
        all_mean_violated.push(count_violated as f64/record_num / params.n as f64 );
        all_mean_report_false.push(count_report_false as f64/record_num / params.n as f64);        
        all_mean_harms.push(total_harms as f64/record_num);
    }
    (all_mean_violated, all_mean_report_false, all_mean_harms)
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_run_n() {
        let mode = Mode::None;
        let (all_mean_violated, all_mean_report_false, all_mean_harms) = run_seeds(mode);
        println!("violated_rate:     {:?}", all_mean_violated);
        println!("report_false_rate: {:?}", all_mean_report_false);
        println!("all_harms_rate:    {:?}", all_mean_harms );
        all_mean_violated.iter().for_each(|value| assert!(*value<=1.0 && *value>=0.95 ));
    }
}

