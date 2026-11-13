use rand::rngs::StdRng;
use rand::{RngExt};
use crate::{Action, Actions, Observes, ObserveStatus, Params, Reaction, Reactions};

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, Clone, PartialEq)]
pub struct Qreport {
    pub report_see: f64,
    pub report_notsee: f64,
    pub ignore_see: f64,
    pub ignore_notsee: f64
}
impl Qreport {
    pub fn new(rng:&mut StdRng) -> Self{
        Qreport{
            report_see: rng.random(),
            report_notsee:rng.random(),
            ignore_see:rng.random(),
            ignore_notsee:rng.random()
        }
    }
}

#[derive(Debug, Clone)]
pub struct Agent{
    pub id: usize,
    pub qvalue: Qvalue,
    pub qreport: Qreport,
    pub count_report: usize,
    pub count_false_report:usize,
    pub count_ignore:usize,
}

impl Agent {
    pub fn new(id: usize, rng: &mut StdRng)  -> Self{
        let qvalue = Qvalue::new(rng);
        let qreport = Qreport::new(rng);
        Agent{id: id, qvalue:qvalue,qreport:qreport, count_report:0, count_false_report:0, count_ignore:0}
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
    pub fn update_qvalue(&mut self, params:&Params, action:&Action, reward_action:f64){
        if action == &Action::Violated {
            self.qvalue.violated_case += (reward_action - self.qvalue.violated_case) * params.learning_rate;
        } else {
            self.qvalue.compliant_case += (reward_action - self.qvalue.compliant_case) * params.learning_rate;
        }
    }

    pub fn get_reaction(&self, params:&Params, observes:&Observes, rng:&mut StdRng) -> Reaction {
        let id = self.id;
        let observe_status = &observes.observes[id].observe_status;
        let greedy_val:f64 = rng.random();
        let reaction = if greedy_val > 1.0 - params.epsilon {
            let random_val:f64 = rng.random();
            if random_val >= 0.5 {
                Reaction::Report
            } else {
                Reaction::Ignore    
            } 
        } else if &ObserveStatus::See == observe_status {
            if self.qreport.report_see >= self.qreport.ignore_see {
                Reaction::Report
            } else {
                Reaction::Ignore
            }
        } else {
            if self.qreport.report_notsee >= self.qreport.ignore_notsee {
                Reaction::Report
            } else {
                Reaction::Ignore
            }
        };
        reaction
    }

    pub fn update_qreport(&mut self, params:&Params, reaction:&Reaction, reward_reaction:f64, observes:&Observes) {
        let id = self.id;
        let observe_status = &observes.observes.iter()
                                                .find(|o| o.observer == id).unwrap()
                                                .observe_status;
        match observe_status {
            ObserveStatus::See => {
                if reaction == &Reaction::Report {
                    self.qreport.report_see += (reward_reaction - self.qreport.report_see) * params.learning_rate;
                } else {
                    self.qreport.ignore_see += (reward_reaction - self.qreport.ignore_see) * params.learning_rate;
                }
            },
            ObserveStatus::Notsee => {
                if reaction == &Reaction::Report {
                    self.qreport.report_notsee += (reward_reaction - self.qreport.report_notsee) * params.learning_rate;
                } else {
                    self.qreport.ignore_notsee += (reward_reaction - self.qreport.ignore_notsee) * params.learning_rate;
                }
            }
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

    pub fn get_reactions(&self, params:&Params, observes:&Observes, rng:&mut StdRng) -> Reactions {
        let reactions:Vec<Reaction> = self.agents.iter()
                    .map(|agent| agent.get_reaction(params, observes, rng))
                    .collect();
        Reactions{reactions:reactions}
    }
}

