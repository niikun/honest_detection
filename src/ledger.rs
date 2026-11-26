
use crate::{Params, Mode, Observes, Observe, ObserveStatus, ReportStates, ReportState};

#[derive(Debug, Clone, Copy,PartialEq)]
pub struct Event{
    pub round:usize,
    pub kind:EventType
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventType {
    Report{observer:usize, observed:usize},
    Reward{who:usize, reward:f64},
    Judge{observer:usize,observed:usize,is_guilty:bool}
}

#[derive(Debug, Clone)]
pub struct Ledger{
    pub events:Vec<Event>
}

impl Event {
    pub fn new(round: usize, kind:EventType) -> Self {
        Self {
            round: round,
            kind:kind
        }
    }
}

impl Ledger{
    pub fn new() -> Self {
        Ledger{
            events:Vec::new()
        }
    }
    pub fn read(&self) -> &[Event] {
        &self.events
    }

    pub fn append(&mut self, event:Event) {
        self.events.push(event)
    }
}

#[derive(Clone, Debug)]
pub struct SimVerifier<'a>{
    all_observes:&'a Vec<Observes>,
}

pub trait Verify {
    fn verify(&self, event:&Event)->Option<bool> ;
}

impl<'a> SimVerifier<'a> {
    pub fn new(all_observes:&'a Vec<Observes>) -> Self {
        Self {
            all_observes: all_observes
        }
    }
}

impl<'a> Verify for SimVerifier<'a> {
    fn verify(&self, event:&Event)-> Option<bool> {
        match event.kind {
            EventType::Report{observer:target_observer, observed:target_observed} =>{
                let observes = &self.all_observes[event.round];
                let mut is_found = false;
                for observe in observes.observes.iter(){
                    if observe.observer == target_observer 
                        && observe.observed == target_observed {
                            if observe.observe_status == ObserveStatus::See{
                                return Some(true);
                            }
                            is_found = true;
                    } 
                }
                if is_found {
                    return Some(false)
                } else {
                    panic!("Event Report do not match Observes");
                }
            },
            _ =>{
                return None;
            }
        }
    }
}

pub fn make_report_event(round:usize, observes:&Observes, report_states:&ReportStates) -> Vec<Event>{
    assert_eq!(observes.observes.len(),report_states.states.len());
    report_states.states.iter().zip(observes.observes.iter())
        .filter(|(state,_observe)|{
            *state == &ReportState::ReportTrue || *state == &ReportState::ReportFalse 
        })
        .map(|(_state,observe)| {
            Event::new(
                round, 
                EventType::Report{observer:observe.observer, observed:observe.observed}
            )
        })
        .collect()
}

pub fn make_judge_event(events:&Vec<Event>, all_observes:&Vec<Observes>) -> Vec<Event>{
    let verifier = SimVerifier::new(all_observes);
    let mut judge_events:Vec<Event> = Vec::new();
    for event in events {
        match event.kind {
            EventType::Report{observer, observed} =>{
                let judgement = verifier.verify(&event);
                match judgement {
                    Some(is_guilty) => {
                        let judege_event = Event::new(
                            event.round, 
                            EventType::Judge{observer:observer,observed:observed,is_guilty:is_guilty});
                        judge_events.push(judege_event);
                    },
                    _ =>{}
                }
            },
            _ =>{}
        }
    }
    judge_events
}

pub fn make_reward_event(round:usize, report_rewards:&Vec<(f64, f64)>, report_states:&ReportStates) -> Vec<Event> {
    let mut results:Vec<Event> = Vec::new(); 
    for (i, ((reward, penalty),state )) in report_rewards.iter().zip(report_states.states.iter()).enumerate(){
        if  penalty != &0.0 {
            let event = Event::new(round, EventType::Reward { who: i, reward: *penalty });
            results.push(event);
        } 
        if state != &ReportState::Ignore {
            let event = Event::new(round, EventType::Reward { who: i, reward: *reward });
            results.push(event);
        }
    }
    results
}    

#[cfg(test)]
mod tests{
    // use rand::rngs::StdRng;
    // use rand::{RngExt, SeedableRng};
    use approx::abs_diff_eq;
    use super::*;
    #[test]
    fn test_read() {
        let event1 = Event::new(
            0,
            EventType::Report{observer:0,observed:1}
        );
        let event2 = Event::new(
            1,
            EventType::Judge{observer:0, observed:1, is_guilty:true}
        );
        let event3 = Event::new(
            2,
            EventType::Reward{who:0, reward:1.1}
        );
        let event4 = Event::new(
            3,
            EventType::Reward{who:1,reward:-2.0}
        );
        let mut ledger = Ledger::new();
        let events = vec![event1, event2, event3, event4];
        events.clone().into_iter().enumerate()
        .for_each(|(i, e)| {
            ledger.append(e);
            for j in 0..=i{
                assert_eq!(ledger.read()[j], events[j]);
            }
        });
        let records = ledger.read();
        // println!("{:?}",records);
    }

    fn set_sim_vec() -> Vec<Observes> {
        let observe1  = Observe {
            observer:0,
            observed:1,
            observe_status:ObserveStatus::See
        };
        let observe2 = Observe {
            observer:1,
            observed:0,
            observe_status:ObserveStatus::Notsee
        };
        let observe3  = Observe {
            observer:0,
            observed:1,
            observe_status:ObserveStatus::See
        };
        let observe4 = Observe {
            observer:1,
            observed:0,
            observe_status:ObserveStatus::Notsee
        };
        let observes1 = Observes {
            observes: vec![observe1, observe2]
        };
        let observes2 = Observes {
            observes: vec![observe3, observe4]
        };
        let all_observes = vec![observes1, observes2];
        all_observes
    }

    #[test]
    fn test_verifier(){
        let sim_vec = set_sim_vec();        
        let sim_verifier = SimVerifier::new(&sim_vec);
        let event1 = Event::new(0,EventType::Report{observer:0,observed:1});
        let event2 = Event::new(1,EventType::Report{observer:1, observed:0});
        let event3 = Event::new(0, EventType::Judge{observer:0, observed:1,is_guilty:true});
        let event4 = Event::new(1, EventType::Reward { who: 0, reward: 1.0 });
        assert_eq!(sim_verifier.verify(&event1), Some(true));
        assert_eq!(sim_verifier.verify(&event2), Some(false));    
        assert_eq!(sim_verifier.verify(&event3), None);    
        assert_eq!(sim_verifier.verify(&event4), None);    
    }
    
    #[should_panic(expected = "Event Report do not match Observes")]
    #[test]
    fn test_verifier_panic_case(){    
        let sim_vec = set_sim_vec();
        let sim_verifier = SimVerifier::new(&sim_vec);      
        let event = Event::new(0, EventType::Report{observer:100, observed:200});  
        sim_verifier.verify(&event);    
    }
    #[test]
    fn test_make_report_event(){
        let observes = &Observes { 
            observes: vec![
                Observe{observer:0, observed:1,observe_status:ObserveStatus::See},
                Observe{observer:1, observed:2,observe_status:ObserveStatus::Notsee},
                Observe{observer:2, observed:0,observe_status:ObserveStatus::See}
                ]
        };
        let report_states = &ReportStates { states: vec![
            ReportState::ReportTrue,
            ReportState::ReportFalse,
            ReportState::Ignore
        ]};
        let events = make_report_event(0, observes, report_states);
        let event1 = Event::new(0,EventType::Report{observer:0,observed:1});
        let event2 = Event::new(0,EventType::Report{observer:1,observed:2});
        let expected = vec![event1, event2];
        assert_eq!(events, expected);
    }
    #[test]
    fn test_make_rewardevent(){
        let mut params = Params::default();
        let round = 0;
        params.mode = Mode::Retro;
        let report_rewards = vec![
            (-&params.cost + &params.r_imm + &params.r_true, 0.0),
            (-&params.cost + &params.r_imm + &params.r_true, 0.0),
            (0.0, -&params.p_viol),
            (-&params.cost + &params.r_imm - &params.p_false, 0.0)
            ]; 
        // let event0 :Event = Event::new(0, EventType::Judge { observer: 0, observed: 2, is_guilty: true });
        // let event1 :Event = Event::new(0, EventType::Judge { observer: 1, observed: 2, is_guilty: true });
        // let event2 :Event = Event::new(0, EventType::Judge { observer: 3, observed: 0, is_guilty: false });
        // let events = vec![event0, event1, event2];
        let report_states = ReportStates {
            states: vec![ReportState::ReportTrue, ReportState::ReportTrue, ReportState::Ignore, ReportState::ReportFalse]
        };
        let results = make_reward_event(round, &report_rewards, &report_states);
        let expected1 = Event::new(0, EventType::Reward{who: 0, reward: -&params.cost + &params.r_imm + &params.r_true});
        let expected2 = Event::new(0, EventType::Reward{who: 2, reward: - &params.p_viol});
        let expected3 = Event::new(0, EventType::Reward{who: 1, reward: -&params.cost + &params.r_imm + &params.r_true});
        let expected4 = Event::new(0, EventType::Reward{who: 3, reward: -&params.cost + &params.r_imm - &params.p_false});
        let expecteds: Vec<Event> = vec![expected1, expected3, expected2, expected4];
        assert_eq!(results.len(), expecteds.len());
        results.iter().zip(expecteds.iter())
            .for_each(|(r, e)| {
                let (result_who, result_reward) = match r.kind {
                    EventType::Reward{who, reward} => (who, reward),
                    _ => panic!("Invalid EventType")
                };
                let (expected_who, expected_reward) = match e.kind {
                    EventType::Reward{who, reward} => (who, reward),
                    _ => panic!("Invalid EventType")
                };
                assert_eq!(result_who, expected_who);
                assert!(abs_diff_eq!(result_reward, expected_reward, epsilon=1e-8));
            })
    }

    #[test]
    fn test_make_judge_event(){
        // ラウンド0の観測: 0 が 1 を See、1 が 0 を Notsee（set_sim_vec）
        let all_observes = set_sim_vec();
        let events = vec![
            Event::new(0, EventType::Report{observer: 0, observed: 1}),
            Event::new(0, EventType::Report{observer: 1, observed: 0}),
            // Report 以外は無視される
            Event::new(0, EventType::Reward{who: 0, reward: 1.0}),
        ];
        let results = make_judge_event(&events, &all_observes);
        let expected = vec![
            Event::new(0, EventType::Judge{observer: 0, observed: 1, is_guilty: true}),
            Event::new(0, EventType::Judge{observer: 1, observed: 0, is_guilty: false}),
        ];
        assert_eq!(results, expected);
    }
}
