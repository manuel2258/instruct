use crate::parse::ast::File;

mod executor;

pub fn execute(file: File, task_name: &str) {
    let mut executor = TaskExecutor::new(file.get_task(task_name).unwrap());

    executor.execute();
}
