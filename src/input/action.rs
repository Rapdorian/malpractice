#[derive(Clone, Debug, Default)]
pub struct ActionState {
    pub value: f32,
    inner: f32,
}

impl ActionState {

    pub fn get(&self) -> f32 {
        self.value
    }

    pub fn set(&mut self, val: f32) {
        self.inner = val;
        self.value = val;
    }

    pub fn normalize(&mut self) {
       self.value = self.inner.max(0.0).min(1.0);
    }

    pub fn press(&mut self){
       self.inner += 1.0;
       self.normalize();
    }

    pub fn release(&mut self){
        self.inner -= 1.0;

        // self.inner should never be negative.
        // If it ever goes negative that means we have likely missed
        // a press() event. (Or we are mixing axis and button inputs
        // which has its own problems)
        self.inner = self.inner.max(0.0);
    }

    pub fn tick(&mut self){
        self.normalize();
    }
}


