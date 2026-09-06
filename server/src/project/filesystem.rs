pub trait Filesystem {
    async fn create_file(&self, path: String, content: &Vec<u8>) -> Result<(), String>;
    async fn create_dir(&self, path: String) -> Result<(), String>;
}
