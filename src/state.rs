use crate::persistence::Persistable;

pub struct State {
    pub switch_1: bool,
    pub switch_2: bool,
}

impl Persistable for State {
    fn from_u8(input: u8) -> Self {
        let switch_1 = (input & (1 << 0)) > 0;
        let switch_2 = (input & (1 << 1)) > 0;

        State { switch_1, switch_2 }
    }
    fn into_u8(&self) -> u8 {
        let mut result = 0u8;
        result &= if self.switch_1 { 1 } else { 0 };
        result &= if self.switch_2 { 1 } else { 0 } << 1;
        return result;
    }
}
