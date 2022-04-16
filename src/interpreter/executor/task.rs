use crate::parse::ast::Task;

use crate::interpreter::command_executor::CommandExecutor;

pub struct TaskExecutor {
    task: Task,
}

impl TaskExecutor {
    pub fn new(task: Task) -> Self {
        Self { task }
    }

    pub fn execute(&mut self) {
        for command in &self.task.exec.cmds {
            let mut executor = CommandExecutor::new(command.clone());
            let (stdout, stderr) = executor.execute().unwrap();
        }
    }
}
