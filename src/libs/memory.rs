
use crate::libs::action_executer::{ActionResult};
use tokio::*;
use chrono::Local;


pub struct Responses{
    pub response : String,
    pub role : String
}

pub struct Memory{
    pub response_history : Vec<Responses>,
    pub execute_result : Vec<ActionResult>
}


impl Memory {
    pub fn default() -> Self {
        let time = Local::now().format("%H:%M:%S").to_string();
        Memory {
            response_history: vec![
                Responses {
                    response : format!("MEMORY STARTED AT : {}", time), 
                    role : "User".to_string()
                }
            ],

            execute_result : vec![ActionResult{
                action : crate::libs::action_executer::Action::Read(format!("")),
                success : true,
                content : "BEGINNING OF EXECUTE HISTORY".to_string()
            }],

        }
    }

    pub fn append_to_history(&mut self, response : Responses) {
        self.response_history.push(response);
    }

    pub fn append_to_result(&mut self, action_result : ActionResult) {
        self.execute_result.push(action_result);
    }
}
