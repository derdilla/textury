pub trait Filesystem {
    fn create_file(&self, path: String, content: &Vec<u8>) -> impl Future<Output = Result<(), String>>;
    fn create_dir(&self, path: String) -> impl Future<Output = Result<(), String>>;
}
