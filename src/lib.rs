use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};

#[derive(Debug, Copy, Clone)]
pub struct Params {
    n:usize,
    gain:f64,
    harm:f64,
    p_viol:f64,
    detect_p:f64,
    cost:f64,
    learning_rate:f64,
    seed:u64,
    epochs:usize,
    epsilon:f64
}

#[derive(Debug, PartialEq)]
pub enum Mode {
    None,
    Immediate,
    Retro
}

impl Default for Params { 
    fn default()->Self {
        Self { n: 10, gain: 1.0, harm: 0.3, p_viol:0.0, detect_p:0.5, cost: 0.2, learning_rate:0.1,seed:64,epochs:1000, epsilon:0.3}
    }
}
#[derive(Debug, PartialEq)]
pub struct Qvalue {
    compliant_case:f64,
    violated_case:f64
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
    report_seen: f64,
    report_notseen: f64,
    ignore_seen: f64,
    ignore_notseen: f64
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
    qvalue: Qvalue,
    qreport: Qreport,
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
}

#[derive(Debug)]
pub struct Agents {
    agents: Vec<Agent>
}

impl Agents {
    pub fn get_actions(&self, params:&Params, rng: &mut StdRng) -> Actions{
        let actions = self.agents.iter()
                    .map(|agent| agent.get_action(params,rng))
                    .collect();
        Actions{actions:actions}
    }
}

#[derive(Debug,PartialEq)]
pub enum Action {
    Compliant,
    Violated
}

pub struct Actions {
    actions: Vec<Action>
}

#[derive(Debug, PartialEq)]
pub enum ObserveStatus {
    See,
    Notsee
}

pub struct Observe {
    observer:usize,
    observed:usize,
    observe_status:ObserveStatus
}

impl Observe {
    pub fn new(rng:&mut StdRng, params:&Params,observer:usize, actions:&Actions) -> Self{ 
        let n = params.n;   
        assert!(n != 1);
        let mut observed = rng.random_range(0..n);
        while observed == observer{
            observed = rng.random_range(0..n);
        } 
        let value_p:f64 = rng.random();
        let observe_status = if (value_p >= (1.0 - params.detect_p)) 
                                                && (actions.actions[observed]==Action::Violated)  {
            ObserveStatus::See
        } else {
            ObserveStatus::Notsee
        };
        Observe{observer:observer, observed:observed, observe_status:observe_status}
    }
}

pub struct Observes {
    observes:Vec<Observe>
}

impl Observes {
    pub fn new(rng:&mut StdRng, params:&Params, actions:&Actions)-> Self{
        let mut observes:Vec<Observe> =Vec::new();
        for i in 0..params.n{
            let observe:Observe = Observe::new(rng, params, i, actions);
            observes.push(observe);
        }
        Observes{observes:observes}
    }
}

pub fn run(params:&Params) ->Qvalue{
    let mut rng = StdRng::seed_from_u64(params.seed);
    let mut agent = Agent::new(&mut rng);
    let mut amount_violated = 0.0;
    let mut count_violated = 0;
    let mut amount_compliant = 0.0;
    let mut count_compliant = 0;
    for i in 0..params.epochs{
        let action = agent.get_action(params, &mut rng);
        agent.update_qvalue(&params,&action, &mut rng);
        if i >= params.epochs / 2 {
            if action == Action::Violated {
                amount_violated += agent.qvalue.violated_case;
                count_violated += 1;            
            } else {
                amount_compliant += agent.qvalue.compliant_case;
                count_compliant += 1;        
            }
        }
        println!("{}:,{:?}", i, agent.qvalue);
    }
    let compliant_case:f64 = amount_compliant / count_compliant as f64;
    let violated_case:f64 = amount_violated / count_violated as f64;
    Qvalue{compliant_case: compliant_case, violated_case: violated_case}
}

pub fn get_reward(params:&Params,action:&Action, rng: &mut StdRng) -> f64 {
    let random_num:f64 = rng.random();
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
        assert_eq!(params.n, 10usize);
        assert_eq!(params.gain, 1.0f64);
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

    #[test]
    fn test_get_actions(){
        let params = Params::default();
        let mut rng =StdRng::seed_from_u64(params.seed);
        let agent1 = Agent::new(&mut rng);
        let agent2 = Agent::new(&mut rng);
        let agents = Agents{agents:vec![agent1, agent2]};
        let actions = agents.get_actions(&params, &mut rng);
        assert_eq!(actions.actions.len(),2);
        assert!(actions.actions[0]==Action::Compliant || actions.actions[0]==Action::Violated);
    }
    #[test]
    fn test_get_observe_detectp1(){
        let mut params = Params::default();
        params.n = 2;
        params.detect_p = 1.0;
        let mut rng = StdRng::seed_from_u64(params.seed);
        let actions = Actions{actions:vec![Action::Compliant, Action::Violated]};
        let observe1 = Observe::new(&mut rng, &params, 0, &actions);
        let observe2 = Observe::new(&mut rng, &params, 1, &actions);
        assert_eq!(observe1.observe_status, ObserveStatus::See);
        assert_eq!(observe2.observe_status, ObserveStatus::Notsee);
    }
        #[test]
    fn test_get_observe_detectp0(){
        let mut params = Params::default();
        params.n = 2;
        params.detect_p = 0.0;
        let mut rng = StdRng::seed_from_u64(params.seed);
        let actions = Actions{actions:vec![Action::Compliant, Action::Violated]};
        let observe1 = Observe::new(&mut rng, &params, 0, &actions);
        let observe2 = Observe::new(&mut rng, &params, 1, &actions);
        assert_eq!(observe1.observe_status, ObserveStatus::Notsee);
        assert_eq!(observe2.observe_status, ObserveStatus::Notsee);
    }

}
