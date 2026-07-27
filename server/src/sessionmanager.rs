use std::collections::HashMap;
use uuid::Uuid;
pub struct SessionManager {
    sessions: HashMap<String, String>,
}
impl SessionManager {
    pub fn new() -> Self {
        Self{sessions:HashMap::new()}
    }
    pub fn create_session(&mut self,username:String) -> String {
        let session_id = Uuid::new_v4().to_string();
        self.sessions.insert(session_id.clone(), username.clone());
        session_id
    }
    pub fn validate(&self,session_id:String) -> bool {
        self.sessions.contains_key(&session_id)
    }
    pub fn get_username(&self,session_id:String) -> Option<String> {
        self.sessions.get(&session_id).cloned()
    }
    pub fn remove_session(&mut self,session_id:String) -> bool {
        self.sessions.remove(&session_id).is_some()
    }
}
