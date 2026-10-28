
#[derive(Debug)]
pub struct Params {
    n:u32,
    gain:f32,
    harm:f32,
    p_viol:f32,
    detect_p:f32,
    cost:f32,
}

#[derive(Debug, PartialEq)]
pub enum Mode {
    None,
    Immediate,
    Retro
}

impl Default for Params { 
    fn default()->Self {
        Self { n: 10, gain: 1.0, harm: 0.3, p_viol:3.0, detect_p:0.5, cost: 0.2}
    }
}


pub fn set_params() {}

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
}
