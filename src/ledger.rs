pub struct Event{
    round:usize,
    kind:EventType
}

pub enum EventType {
    Report{observer:usize, observed:usize},
    Reward{who:usize, reward:f64},
    Judge{observer:usize,observed:usize,is_guilty:bool}
}
