use crate::parse::ast::Namespace;

mod executor;
mod interpolateable;
mod stack;

pub fn execute(file: Namespace, task_name: &str) {
    //let mut executor = TaskExecutor::new(file.get_task(task_name).unwrap());

    //    executor.execute();
}
