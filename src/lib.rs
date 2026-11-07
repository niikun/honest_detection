use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
mod agent;
use agent::{Agent,Agents, Qvalue, Qreport};

#[derive(Debug, Copy, Clone)]
pub struct Params {
    n:usize,
    gain:f64, //違反時の報酬
    harm:f64, //違反者から被る損害
    p_viol:f64, //違反時の罰則
    detect_p:f64,  //違反の検知確率
    cost:f64,  //報告のコスト
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

#[derive(Debug,PartialEq)]
pub enum Action {
    Compliant,
    Violated
}

pub struct Actions {
    actions: Vec<Action>
}

#[derive(Debug,PartialEq, Copy, Clone)]
pub enum Reaction { 
    Report,
    Ignore
} 
pub struct Reactions {
    reactions: Vec<Reaction>

}

#[derive(Debug, PartialEq)]
pub enum ObserveStatus {
    See,
    Notsee
}
#[derive(Debug)]
pub struct Observe {
    pub observer:usize,
    pub observed:usize,
    pub observe_status:ObserveStatus
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
    let mut agent = Agent::new(0usize,&mut rng);
    let mut amount_violated = 0.0;
    let mut count_violated = 0;
    let mut amount_compliant = 0.0;
    let mut count_compliant = 0;
    for i in 0..params.epochs{
        let action = agent.get_action(params, &mut rng);
        let (reward_action, _ ) = get_reward(params, &action, &Reaction::Ignore, &mut rng);
        agent.update_qvalue(&params,&action, reward_action);
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

pub fn get_reward(params:&Params,action:&Action, reaction:&Reaction, rng: &mut StdRng) -> (f64, f64) {
    let random_num:f64 = rng.random();
    let reward_action = if action == &Action::Violated {
        if random_num <= params.detect_p {
            params.gain - params.p_viol
        } else { 
            params.gain
        }
    } else {
        0.0
    };
    //未実装
    let reward_reaction = 0.0;
    (reward_action, reward_reaction)
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
        let agent1 = Agent::new(0, &mut rng);
        let agent2 = Agent::new(1, &mut rng);
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

    #[test]
    fn test_get_reactions(){
        let mut params = Params::default();
        params.n = 4;
        params.epsilon = 0.0;
        let mut rng = StdRng::seed_from_u64(params.seed);
        let mut agent1 = Agent::new(0, &mut rng);
        let mut agent2 = Agent::new(1, &mut rng);
        let mut agent3 = Agent::new(2, &mut rng);
        let mut agent4 = Agent::new(3, &mut rng);
        agent1.qreport.report_see = 1.0;
        agent2.qreport.ignore_see = 1.0;
        agent3.qreport.report_notsee = 1.0;
        agent4.qreport.ignore_notsee = 1.0;
        let agents = Agents{agents:vec![agent1, agent2, agent3, agent4]};
        let actions = Actions{
            actions:vec![Action::Violated, Action::Violated, Action::Compliant, Action::Compliant]
        };
        let mut observes = Observes::new(&mut rng, &params, &actions);
        observes.observes[0].observe_status = ObserveStatus::See;
        observes.observes[1].observe_status = ObserveStatus::See;
        observes.observes[2].observe_status = ObserveStatus::Notsee;
        observes.observes[3].observe_status = ObserveStatus::Notsee;
        let reactions = agents.get_reactions(&params, &observes, &mut rng);
        assert_eq!(reactions.reactions.len(), 4);
        assert_eq!(reactions.reactions[0], Reaction::Report);
        assert_eq!(reactions.reactions[1], Reaction::Ignore);
        assert_eq!(reactions.reactions[2], Reaction::Report);
        assert_eq!(reactions.reactions[3], Reaction::Ignore);
    }

    #[test]
    fn test_update_qreport(){
        let mut agent = Agent::new(0, &mut StdRng::seed_from_u64(64));
        agent.qreport.report_see = 0.0;
        agent.qreport.ignore_see = 0.0;
        agent.qreport.report_notsee = 0.0;
        agent.qreport.ignore_notsee = 0.0;    
        let mut params = Params::default();
        params.learning_rate = 0.1;
        params.n = 2;
        let observes = Observes{observes:vec![
            Observe{observer:0, observed:1, observe_status:ObserveStatus::See},
            Observe{observer:1, observed:0, observe_status:ObserveStatus::Notsee}
        ]};
        let mut rng = StdRng::seed_from_u64(params.seed);
        let actions = Actions{actions:vec![Action::Compliant, Action::Violated]};
        let reactions = vec![Reaction::Report, Reaction::Ignore];
        agent.update_qreport(&params, &reactions[0], 1.0, &observes);
        let qreport = &agent.qreport;
        assert_eq!(qreport.report_see, 0.1);
        assert_eq!(qreport.ignore_see, 0.0);
        assert_eq!(qreport.report_notsee, 0.0);
        assert_eq!(qreport.ignore_notsee, 0.0);   
    }
}
