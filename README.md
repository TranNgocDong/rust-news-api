Rust News API
Yêu cầu: Rust Edition 2024, PostgreSQL chạy tại `localhost:5432`.
1. Tạo database
Chạy trong pgAdmin/psql:
```sql
CREATE DATABASE news_api;
```
2. Cấu hình `.env`
```powershell
Copy-Item .env.example .env
```
Điền thông tin trong `.env`:
```env
SERVER_HOST=127.0.0.1
SERVER_PORT=8081
DATABASE_URL=postgres://postgres:YOUR_PASSWORD@localhost:5432/news_api
JWT_SECRET=replace_with_secret_at_least_32_characters
UPLOAD_DIR=uploads
MAX_UPLOAD_BYTES=2097152
```
3. Chạy server
```powershell
cargo check
cargo run
```
Migrations tự chạy khi khởi động; health check:
```powershell
curl.exe http://127.0.0.1:8081/health
```
Swagger UI: http://127.0.0.1:8081/swagger-ui
4. Test API
```powershell
$base = "http://127.0.0.1:8081"
curl.exe -X POST "$base/api/auth/register" -H "Content-Type: application/json" -d '{"username":"user1","password":"secret123"}'
$auth = curl.exe -s -X POST "$base/api/auth/login" -H "Content-Type: application/json" -d '{"username":"user1","password":"secret123"}' | ConvertFrom-Json
$token = $auth.data.token
curl.exe "$base/api/news"
curl.exe "$base/api/news?q=rust&page=1&limit=10"
curl.exe "$base/api/users/me" -H "Authorization: Bearer $token"
```
5. Cấp quyền admin
Đăng ký trước rồi chạy SQL:
```sql
UPDATE users SET role = 'admin', is_manager = TRUE WHERE username = 'user1';
```
Đăng nhập lại để lấy token mới; admin/manager được CRUD tin tức.
6. Lệnh hữu ích
```powershell
cargo fmt
cargo clippy
cargo build --release
```
Lưu ý: `JWT_SECRET` phải dài ít nhất 32 ký tự. Swagger Authorize chỉ dán token, không thêm `Bearer`. Nếu cổng 8081 bận, tắt server cũ hoặc đổi `SERVER_PORT`.