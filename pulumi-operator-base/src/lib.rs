use springtime_di::instance_provider::ComponentInstancePtr;

pub mod stack;

pub type Inst<T> = ComponentInstancePtr<T>;
