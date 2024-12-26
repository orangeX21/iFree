use rusqlite::Connection;
use std::env;
use std::path::PathBuf;

fn main() {
    let home_dir = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .unwrap();
    let db_path = if cfg!(target_os = "windows") {
        PathBuf::from(home_dir).join(r"AppData\Roaming\Cursor\User\globalStorage\state.vscdb")
    } else {
        PathBuf::from(home_dir)
            .join("Library/Application Support/Cursor/User/globalStorage/state.vscdb")
    };

    match Connection::open(&db_path) {
        Ok(conn) => {
            match conn.query_row(
                "SELECT value FROM ItemTable WHERE key = 'cursorAuth/accessToken'",
                [],
                |row| row.get::<_, String>(0),
            ) {
                Ok(token) => println!("访问令牌: {}", token.trim()),
                Err(err) => eprintln!("获取令牌时出错: {}", err),
            }
        }
        Err(err) => eprintln!("无法打开数据库: {}", err),
    }
}
