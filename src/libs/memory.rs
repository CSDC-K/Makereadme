
use crate::libs::{action_executer::ActionResult, build::OptimizationLevel, optlib::OptProfile};
use chrono::Local;


pub struct HistoryEntry{
    pub content : String,
    pub role : String,
}

pub struct Memory{
    pub history : Vec<HistoryEntry>,
    pub execute_result : Vec<ActionResult>,
    pub agentic_notes : Vec<String>,
    pub opt_profile : OptProfile,
}



impl Memory {
    pub fn default(opt_profile: OptProfile) -> Self {
        let time = Local::now().format("%H:%M:%S").to_string();
        Memory {
            history: vec![
                HistoryEntry {
                    content : format!("MEMORY STARTED AT : {}", time), 
                    role : "User".to_string(),
                }
            ],

            execute_result : vec![ActionResult{
                action : crate::libs::action_executer::Action::Read(format!("")),
                success : true,
                content : "BEGINNING OF EXECUTE HISTORY".to_string()
            }],

            agentic_notes : vec![],
            opt_profile,
        }
    }

    pub fn append_to_history(&mut self, entry : HistoryEntry) {
        
        let row_to_be_removed = self.opt_profile.history_limit - 3;

        if self.history.len() >= self.opt_profile.history_limit {
            self.history.remove(row_to_be_removed);
        }

        self.history.push(entry);
    }

    pub fn append_to_result(&mut self, action_result : ActionResult) {

        let row_to_be_removed = self.opt_profile.result_history_limit - 3;

        if self.execute_result.len() >= self.opt_profile.result_history_limit {
            self.execute_result.remove(row_to_be_removed);
        }

        self.execute_result.push(action_result);
    }

    pub fn append_to_agentic_notes(&mut self, note : String) {
        self.agentic_notes.push(note);
    }
}
