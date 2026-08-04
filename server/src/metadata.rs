use common::models::{Access::{self, Public}, FileMetadata};
use tokio::fs;
use serde::{Deserialize, Serialize};


const METADATA_PATH: &str = "storage/metadata.json";
#[derive(Debug, Deserialize, Serialize,Clone)]

pub struct MetadataManager {
    metadata: Vec<FileMetadata>,
}

impl MetadataManager {
    pub async fn new() -> Self {
        let metadata = Self::load().await;

        Self { metadata }
    }

    async fn load() -> Vec<FileMetadata> {
        match fs::read_to_string(METADATA_PATH).await {
            Ok(data) => serde_json::from_str(&data).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    async fn save(&self) {
        let json = serde_json::to_string_pretty(&self.metadata).unwrap();

        fs::write(METADATA_PATH, json).await.unwrap();
    }

    pub async fn add(&mut self, file: FileMetadata) {
        self.metadata.push(file);

        self.save().await;
    }

    pub async fn remove(&mut self, owner: &str, filename: &str) {
        for (i,file) in &mut self.metadata.iter().enumerate(){
            if file.owner == owner && file.filename == filename {
                self.metadata.remove(i);
                break;
            }
        }
        self.save().await;
    }

    pub async fn rename(&mut self, owner: &str, old_name: &str, new_name: &str) {
        for file in &mut self.metadata {
            if file.owner == owner && file.filename == old_name {
                file.filename = new_name.to_string();
            }
        }
        self.save().await;
    }

    pub fn find(&self, owner: &str, filename: &str) -> Option<FileMetadata> {
        for file in &self.metadata {
            if file.owner == owner && file.filename == filename {
                return Some(file.clone());
            }
        }
        None
    }


    pub fn get_user_files(&self, owner: &str) -> Vec<FileMetadata> {
        let mut files = Vec::new();
        for file in &self.metadata {
            if file.owner == owner {
                
                files.push(file.clone());
            }
        }
        files
    }
    pub async fn search(&self, filename: &str,user: &str) -> Vec<FileMetadata> {
        let files =self.get_user_files(user);
        let mut target = Vec::new();
        for file in files {
            if file.filename.to_lowercase().contains(filename.to_lowercase().as_str()) {
                target.push(file.clone());
            }
        }
        target
    }

    pub fn update_access(&mut self,filename:&str,owner:&str,access:Access)   {
        for file in &mut self.metadata{
            if file.filename == filename && file.owner == owner.to_string(){
                file.access = access;
                return;
            }
        }
    }
    pub async fn public_files(&self) -> Vec<FileMetadata> {
        let mut files = Vec::new();
        for file in &self.metadata {
            match file.access {
                Access::Public => {
                    files.push(file.clone());
                }
                _ => {}
            }
        }
        files
    }
    pub async fn downloadable(&self , filename:&str , owner:&str,is_owner:bool) -> bool{
        let mut files = self.get_user_files(owner);
        files.extend_from_slice(&self.public_files().await);
        for file in files{
            if file.owner==owner && file.filename == filename{
                if is_owner{
                    return true;
                }
                match file.access {
                    Access::Private =>{
                        return false;
                    }
                    Access::Public => {
                        return true;
                    }
                }
            }
        }
        false
    }
}

