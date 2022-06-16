use crossbeam_channel::{RecvError, SendError};
use thiserror::Error;

use crate::util::channel::TwoWayChannel;

use super::message::{action, result, RunnerAction, RunnerRequest, RunnerResponse};

#[derive(Error, Debug)]
pub enum RunnerInterfaceError {
    #[error("received invalid runner response {1:?} for action '{0:?}'")]
    InvalidResponse(&'static str, RunnerResponse),
    #[error("could not send message to channel: {0}")]
    SendChannelError(SendError<RunnerRequest>),
    #[error("could not receive message from channel: {0}")]
    ReceiveChannelError(RecvError),
    #[error("runner could not find command '{0}'")]
    CommandNotFound(String),
}

type RunnerInterfaceResult<T> = std::result::Result<T, RunnerInterfaceError>;

pub struct RunnerInterface {
    channel: TwoWayChannel<RunnerRequest, RunnerResponse>,
}

impl RunnerInterface {
    pub fn new(channel: TwoWayChannel<RunnerRequest, RunnerResponse>) -> Self {
        Self { channel }
    }

    fn send_and_receive(&self, msg: RunnerRequest) -> RunnerInterfaceResult<RunnerResponse> {
        self.channel
            .tx
            .send(msg)
            .map_err(|err| RunnerInterfaceError::SendChannelError(err))?;
        Ok(self
            .channel
            .rx
            .recv()
            .map_err(|err| RunnerInterfaceError::ReceiveChannelError(err))?)
    }

    pub fn run(
        &self,
        runner_name: String,
        command: String,
        trim_stdout: bool,
        trim_stderr: bool,
    ) -> RunnerInterfaceResult<result::RunResult> {
        let msg = RunnerRequest {
            runner_name,
            action: RunnerAction::Run(action::RunAction {
                command,
                trim_stdout,
                trim_stderr,
            }),
        };
        let response = self.send_and_receive(msg)?;

        match response {
            RunnerResponse::Output(output) => Ok(output),
            RunnerResponse::CommandNotFound(command) => {
                Err(RunnerInterfaceError::CommandNotFound(command))
            }
            other_response => Err(RunnerInterfaceError::InvalidResponse("run", other_response)),
        }
    }
}
