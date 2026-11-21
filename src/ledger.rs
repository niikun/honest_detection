
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

#[cfg(test)]
mod tests{
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


}