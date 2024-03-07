use crate::persistence::Persistable;

pub struct State {
    switch_1: bool,
    switch_2: bool,
}

impl Persistable for State {
    fn from(input: u8) -> Self {
        let switch_1 = (input & (1 << 0)) > 0;
        let switch_2 = (input & (1 << 1)) > 0;

        State { switch_1, switch_2 }
    }
    fn into(&self) -> u8 {
        let mut result = 0u8;
        result &= if self.switch_1 { 1 } else { 0 };
        result &= if self.switch_2 { 1 } else { 0 } << 1;
        return result;
    }
}
