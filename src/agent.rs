use rand::rngs::StdRng;
use rand::{RngExt};
use crate::{Params, Action, Actions, get_reward};

#[derive(Debug, PartialEq)]
pub struct Qvalue {
    pub compliant_case:f64,
    pub violated_case:f64
}

impl Qvalue{
    pub fn new(rng:&mut StdRng)->Self {
        Qvalue{
            compliant_case: rng.random(),
            violated_case: rng.random()
        }
    } 
}

#[derive(Debug)]
pub struct Qreport {
    pub report_seen: f64,
    pub report_notseen: f64,
    pub ignore_seen: f64,
    pub ignore_notseen: f64
}
impl Qreport {
    pub fn new(rng:&mut StdRng) -> Self{
        Qreport{
            report_seen: rng.random(),
            report_notseen:rng.random(),
            ignore_seen:rng.random(),
            ignore_notseen:rng.random()
        }
    }
}

#[derive(Debug)]
pub struct Agent{
    pub qvalue: Qvalue,
    pub qreport: Qreport,
    count_report: usize,
    count_false_report:usize,
    count_ignore:usize,
}

impl Agent {
    pub fn new(rng: &mut StdRng)  -> Self{
        let qvalue = Qvalue::new(rng);
        let qreport = Qreport::new(rng);
        Agent{qvalue:qvalue,qreport:qreport, count_report:0, count_false_report:0, count_ignore:0}
    }

    pub fn get_action(&self, params:&Params, rng: &mut StdRng) -> Action{
        let greedy_val:f64 = rng.random();
        let action = if greedy_val > 1.0 - params.epsilon {
            let random_val:f64 = rng.random();
            if random_val >= 0.5 {
                Action::Compliant
            } else {
                Action::Violated
            }
        } else {
            if self.qvalue.compliant_case > self.qvalue.violated_case {
                Action::Compliant
            } else {
                Action::Violated
            }
        };
        action
    }
    pub fn update_qvalue(&mut self, params:&Params, action:&Action, rng:&mut StdRng){
        let reward = get_reward(params,action,rng);
        if action == &Action::Violated {
            self.qvalue.violated_case += (reward - self.qvalue.violated_case) * params.learning_rate;
        } else {
            self.qvalue.compliant_case += (reward - self.qvalue.compliant_case) * params.learning_rate;
        }
    }

    pub fn report(&mut self) {
        self.count_report += 1;
    }

    pub fn false_report(&mut self) {
        self.count_false_report += 1;
    }

    pub fn ignore(&mut self) {
        self.count_ignore += 1;
    }
    pub fn count_report(&self)->usize {
        self.count_report
    }
    pub fn count_false_report(&self)->usize {
        self.count_false_report
    }

    pub fn count_ignore(&self)->usize {
        self.count_ignore
    }


}

#[derive(Debug)]
pub struct Agents {
    pub agents: Vec<Agent>
}

impl Agents {
    pub fn get_actions(&self, params:&Params, rng: &mut StdRng) -> Actions{
        let actions = self.agents.iter()
                    .map(|agent| agent.get_action(params,rng))
                    .collect();
        Actions{actions:actions}
    }
}
