use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

#[derive(Debug, Copy, Clone)]
pub struct Params {
    n:u32,
    gain:f32,
    harm:f32,
    p_viol:f32,
    detect_p:f32,
    cost:f32,
    learning_rate:f32,
    seed:u64,
    epochs:usize
}

#[derive(Debug, PartialEq)]
pub enum Mode {
    None,
    Immediate,
    Retro
}

impl Default for Params { 
    fn default()->Self {
        Self { n: 10, gain: 1.0, harm: 0.3, p_viol:0.0, detect_p:0.5, cost: 0.2, learning_rate:0.1,seed:64,epochs:1000}
    }
}
#[derive(Debug, PartialEq)]
pub struct Qvalue {
    compliant_case:f32,
    violated_case:f32
}

impl Qvalue{
    pub fn new(rng:&mut StdRng)->Self {
        Qvalue{
            compliant_case: rng.random(),
            violated_case: rng.random()
        }
    } 
}

pub struct Agent{
    qvalue: Qvalue,
    count_report: usize,
    count_false_report:usize,
    count_ignore:usize
}

impl Agent {
    pub fn new(rng: &mut StdRng)  -> Self{
        let qvalue = Qvalue::new(rng);
        Agent{qvalue:qvalue,count_report:0, count_false_report:0, count_ignore:0}
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
}

pub struct Agents {
    agents: Vec<Agent>
}

#[derive(Debug,PartialEq)]
pub enum Action {
    Compliant,
    Violated
}

pub fn run(params:&Params) ->Qvalue{
    let mut rng = StdRng::seed_from_u64(params.seed);
    let mut qvalue = Qvalue::new(&mut rng);
    let mut amount_violated = 0.0;
    let mut amount_compliant = 0.0;
    let mut count_violated = 0;
    let mut count_compliant = 0;
    for i in 0..params.epochs{
        let action = get_action(&qvalue, &mut rng);
        update_qvalue(&mut qvalue, &params,&action, &mut rng);
        if i >= params.epochs / 2 {
            if action == Action::Violated {
                amount_violated += qvalue.violated_case;
                count_violated += 1;            
            } else {
                amount_compliant += qvalue.compliant_case;
                count_compliant += 1;        
            }
        }
        println!("{}:,{:?}",i,qvalue);
    }
    let compliant_case:f32 = amount_compliant / count_compliant as f32;
    let violated_case:f32 = amount_violated / count_violated as f32;
    Qvalue{compliant_case: compliant_case, violated_case: violated_case}
}

pub fn update_qvalue(qvalue:&mut Qvalue, params:&Params, action:&Action, rng:&mut StdRng){
    let reward = get_reward(params,action,rng);
    if action == &Action::Violated {
        qvalue.violated_case += (reward - qvalue.violated_case) * params.learning_rate;
    } else {
        qvalue.compliant_case += (reward - qvalue.compliant_case) * params.learning_rate;
    }
}

pub fn get_reward(params:&Params,action:&Action, rng: &mut StdRng) -> f32 {
    let random_num:f32 = rng.random();
    let reward = if action == &Action::Violated {
        if random_num <= params.detect_p {
            params.gain - params.p_viol
        } else { 
            params.gain
        }
    } else {
        0.0
    };
    reward
}

pub fn get_action(qvalue:&Qvalue, rng: &mut StdRng) -> Action{
    let greedy_val:f32 = rng.random();
    let epsilon = 0.3;
    let action = if greedy_val > 1.0 - epsilon {
        let random_val:f32 = rng.random();
        if random_val >= 0.5 {
            Action::Compliant
        } else {
            Action::Violated
        }
    } else {
        if qvalue.compliant_case > qvalue.violated_case {
            Action::Compliant
        } else {
            Action::Violated
        }
    };
    action
}

pub fn choose_action(qvalue:&Qvalue)->Action {
    let action = if qvalue.violated_case >= qvalue.compliant_case {
            Action::Violated
        } else {
            Action::Compliant
        };
    action
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let params = Params::default();
        let mode:Mode = Mode::None;
        assert_eq!(params.n, 10u32);
        assert_eq!(params.gain, 1.0f32);
        assert_eq!(mode, Mode::None);
    }

    #[test]
    fn test_run() {
        let params1 = Params::default();
        let mut params2:Params = params1.clone();
        let mut params3:Params = params1.clone();
        params2.p_viol = 1.5;
        params3.p_viol = 2.5;
        let qvalue1 = run(&params1);
        let qvalue2 = run(&params2);
        let qvalue3 = run(&params3);
        let action1 = choose_action(&qvalue1);
        let action2 = choose_action(&qvalue2);
        let action3 = choose_action(&qvalue3);
        assert_eq!(action1, Action::Violated);
        assert_eq!(action2, Action::Violated);
        assert_eq!(action3, Action::Compliant);
        assert!(qvalue1.violated_case < 1.0);
    }
}
