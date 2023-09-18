use springtime_di::instance_provider::ComponentInstancePtr;

pub mod config_provider;
pub mod kubernetes;
pub mod stack;

pub fn bind() {}

pub type Inst<T> = ComponentInstancePtr<T>;
