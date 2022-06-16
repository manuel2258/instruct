use std::collections::HashMap;

use super::{
    handler::{create_new, DynRunnerHandler},
    message::{action, RunnerAction, RunnerRequest, RunnerResponse},
};

pub struct Router {
    handlers: HashMap<String, DynRunnerHandler>,
}

impl Router {
    pub fn new() -> Self {
        let mut router = Self {
            handlers: HashMap::new(),
        };

        let result = router.handle_request(RunnerRequest {
            runner_name: "default".into(),
            action: RunnerAction::Create(action::CreateAction {
                runner_name: "default".into(),
                runner_type: "command".into(),
                args: HashMap::new(),
            }),
        });

        assert_eq!(result, RunnerResponse::Created);

        router
    }

    pub fn handle_request(&mut self, request: RunnerRequest) -> RunnerResponse {
        if let RunnerAction::Create(create_action) = &request.action {
            if self.handlers.contains_key(&request.runner_name) {
                return RunnerResponse::RunnerAlreadyExists(request.runner_name);
            }

            let new_handler = match create_new(&create_action.runner_type) {
                Some(val) => val,
                None => {
                    return RunnerResponse::RunnerTypeNotExisting(create_action.runner_type.clone())
                }
            };

            assert!(self
                .handlers
                .insert(request.runner_name.clone(), new_handler)
                .is_none());
        }

        let handler = match self.handlers.get_mut(&request.runner_name) {
            Some(val) => val,
            None => return RunnerResponse::RunnerNotExisting(request.runner_name),
        };

        handler.handle(request.action)
    }
}
