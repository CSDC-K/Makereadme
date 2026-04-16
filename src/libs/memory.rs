
use crate::libs::action_executer::{ActionResult};
use chrono::Local;


pub struct HistoryEntry{
    pub content : String,
    pub role : String
}

pub struct Memory{
    pub history : Vec<HistoryEntry>,
    pub execute_result : Vec<ActionResult>
}


impl Memory {
    pub fn default() -> Self {
        let time = Local::now().format("%H:%M:%S").to_string();
        Memory {
            history: vec![
                HistoryEntry {
                    content : format!("MEMORY STARTED AT : {}", time), 
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

    pub fn append_to_history(&mut self, entry : HistoryEntry) {
        self.history.push(entry);
    }

    pub fn append_to_result(&mut self, action_result : ActionResult) {
        self.execute_result.push(action_result);
    }
}
