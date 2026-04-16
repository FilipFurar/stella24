pub enum Condition {
    Comparison,
    Logical,
    Between,
    Like,
    In,
    IsNull,
    IsNotNull,
}

pub struct Check {
    pub name: String,
    pub condition: Condition,
    
}