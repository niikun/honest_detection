
use crate::{Params, Observes, Observe, ObserveStatus};

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
    pub fn new(round: usize, event_type:EventType) -> Self {
        Self {
            round: round,
            kind:event_type
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

pub struct SimVerifier{
    all_observes:Vec<Observes>,
}

pub trait Verify {
    fn verify(&self, event:&Event)->Option<bool> ;
}

impl SimVerifier {
    pub fn new(all_observes:Vec<Observes>) -> Self {
        Self {
            all_observes: all_observes
        }
    }
}

impl Verify for SimVerifier {
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

#[cfg(test)]
mod tests{
    use rand::rngs::StdRng;
    use rand::{RngExt, SeedableRng};
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

    fn set_sim_verifier() -> SimVerifier {
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
        SimVerifier::new(all_observes)
    }

    #[test]
    fn test_verifier(){
        let sim_verifier = set_sim_verifier();        
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
        let sim_verifier = set_sim_verifier();        
        let event = Event::new(0, EventType::Report{observer:100, observed:200});  
        sim_verifier.verify(&event);    
    }
}