
use rand;

#[derive(Debug)]
pub struct Params {
    n:u32,
    gain:f32,
    harm:f32,
    p_viol:f32,
    detect_p:f32,
    cost:f32,
    learning_rate:f32
}

#[derive(Debug, PartialEq)]
pub enum Mode {
    None,
    Immediate,
    Retro
}

impl Default for Params { 
    fn default()->Self {
        Self { n: 10, gain: 1.0, harm: 0.3, p_viol:3.0, detect_p:0.5, cost: 0.2, learning_rate:0.1}
    }
}
#[derive(Debug, PartialEq)]
pub struct Qvalue {
    compliant_case:f32,
    violated_case:f32
}


impl Qvalue{
    pub fn new()->Self {
        Qvalue{
            compliant_case: rand::random(),
            violated_case: rand::random()
        }
    } 
}
#[derive(Debug,PartialEq)]
pub enum Action {
    Compliant,
    Violated
}

pub fn run() ->Qvalue{
    let mut qvalue = Qvalue::new();
    let params = Params::default();
    for i in 0..1000{
        let action = get_action(&qvalue);
        update_qvalue(&mut qvalue, &params,&action);
        println!("{}:,{:?}",i,qvalue);
    }
    qvalue
}

pub fn update_qvalue(qvalue:&mut Qvalue, params:&Params, action:&Action){
    let reward = get_reward(params,action);
    if action == &Action::Violated {
        qvalue.violated_case += (reward - qvalue.violated_case) * params.learning_rate;
    } else {
        qvalue.compliant_case += (reward - qvalue.compliant_case) * params.learning_rate;
    }
}

pub fn get_reward(params:&Params,action:&Action) -> f32 {
    let mut reward = 0.0;
    if action == &Action::Violated {
        reward = params.gain;
    }
    reward
}

pub fn get_action(qvalue:&Qvalue) -> Action{
    let greedy_val:f32 = rand::random();
    let epsilon = 0.3;
    let mut action = Action::Compliant;
    if greedy_val > 1.0 - epsilon {
        let random_val:f32 = rand::random();
        if random_val >= 0.5 {
            action = Action::Compliant;
        } else {
            action = Action::Violated;
        }
    } else {
        if qvalue.compliant_case > qvalue.violated_case {
            action = Action::Compliant;
        } else {
            action = Action::Violated;
        }
    }
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
        let qvalue = run();
        let action = if qvalue.violated_case >= qvalue.compliant_case {
            Action::Violated
        } else {
            Action::Compliant
        };
        assert_eq!(action, Action::Violated);
        assert!(qvalue.violated_case < 1.0);
    }
}
