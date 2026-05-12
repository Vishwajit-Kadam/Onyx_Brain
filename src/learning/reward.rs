#[derive(Debug, Clone, Copy)]
pub struct Reward {
    pub value: f32,
}

impl Reward {
    pub fn success() -> Self {
        Self { value: 1.0 }
    }

    pub fn failure() -> Self {
        Self { value: -1.0 }
    }
}
