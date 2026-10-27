use std::env;

const DEFAULT_SERVER_HOST: &str = "127.0.0.1";
const DEFAULT_SERVER_PORT: u16 = 8081;
const DEFAULT_UPLOAD_DIR: &str = "uploads";
const DEFAULT_MAX_UPLOAD_BYTES: usize = 2 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct Config {
    pub server_host: String,
    pub server_port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub upload_dir: String,
    pub max_upload_bytes: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        dotenvy::dotenv().ok();

        let server_host =
            env::var("SERVER_HOST").unwrap_or_else(|_| DEFAULT_SERVER_HOST.to_string());

        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| DEFAULT_SERVER_PORT.to_string())
            .parse::<u16>()
            .map_err(|_| "SERVER_PORT phải là số nguyên từ 0 đến 65535".to_string())?;

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "Thiếu DATABASE_URL trong file .env".to_string())?;

        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| "Thiếu JWT_SECRET trong file .env".to_string())?;

        if jwt_secret.len() < 32 {
            return Err("JWT_SECRET phải có ít nhất 32 ký tự".to_string());
        }

        let upload_dir =
            env::var("UPLOAD_DIR").unwrap_or_else(|_| DEFAULT_UPLOAD_DIR.to_string());

        let max_upload_bytes = match env::var("MAX_UPLOAD_BYTES") {
            Ok(value) => value
                .parse::<usize>()
                .map_err(|_| "MAX_UPLOAD_BYTES phải là số nguyên dương".to_string())?,
            Err(_) => DEFAULT_MAX_UPLOAD_BYTES,
        };

        Ok(Self {
            server_host,
            server_port,
            database_url,
            jwt_secret,
            upload_dir,
            max_upload_bytes,
        })
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}