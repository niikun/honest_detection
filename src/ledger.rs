
#[derive(Debug, Clone, PartialEq)]
pub struct Event{
    pub round:usize,
    pub kind:EventType
}

#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    Report{observer:usize, observed:usize},
    Reward{who:usize, reward:f64},
    Judge{observer:usize,observed:usize,is_guilty:bool}
}

#[derive(Debug, Clone)]
pub struct Events{
    events:Vec<Event>
}

impl Event {
    pub fn new(round: usize, event_type:EventType) -> Self {
        Self {
            round: round,
            kind:event_type
        }
    }
}

impl Events{
    pub fn read(&self) -> &[Event] {
        let events_vec: Vec<&Event> = self.events.iter()
                .map(|event| event.clone())
                .collect();
        &events_vec
    }

    pub fn append(&mut self, event:Event) {
        self.events.push(event)
    }

    fn write() {

    }
}