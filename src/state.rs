use crate::persistence::Persistable;

pub struct State {
    pub bypass: bool,
}

impl State {
    pub fn new(switch_1: bool) -> Self {
        State { bypass: switch_1 }
    }
}

impl Persistable for State {
    fn from_u8(input: u8) -> Self {
        let switch_1 = (input & (1 << 0)) > 0;

        State { bypass: switch_1 }
    }
    fn into_u8(&self) -> u8 {
        let mut result = 0u8;
        result |= if self.bypass { 1 } else { 0 };
        return result;
    }
}
