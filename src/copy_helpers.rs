use crate::*;

pub trait ValueClone {
    fn clone_box(&self) -> Box<dyn Value>;
}

impl<T> ValueClone for T 
where
    T: 'static + Value + Clone
{
    fn clone_box(&self) -> Box<dyn Value> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Value> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait InstructionClone {
    fn clone_box(&self) -> Box<dyn Instruction>;
}

impl<T> InstructionClone for T
where
    T: 'static + Instruction + Clone
{
    fn clone_box(&self) -> Box<dyn Instruction> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Instruction> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

