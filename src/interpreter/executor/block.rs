use async_trait::async_trait;
use std::collections::HashMap;

use super::{ExecuteResult, Executor};

pub struct BlockExecutor {
    input: Vec<Box<dyn Executor>>,
}

impl BlockExecutor {
    pub fn new(input: Vec<Box<dyn Executor>>) -> Self {
        Self { input }
    }
}

#[async_trait(?Send)]
impl Executor for BlockExecutor {
    async fn execute(&mut self) -> ExecuteResult {
        for executor in self.input.drain(..) {
            executor.execute().await?;
        }
        Ok(HashMap::new())
    }
}
