use std::{
    io::{Read, Write},
    net::TcpListener,
    str::from_utf8,
};

const INDEX: &str = r#"
<h1>Todos</h1>
<form action="/create" method="post">
  <label for="todo">Add todo note:</label>
  <input type="text" id="todo" name="todo">
  <input type="submit" value="Submit">
</form>
"#;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn respond(message: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        message.len(),
        message
    )
    .as_bytes()
    .to_vec()
}

fn index(todos: &[Todo]) -> String {
    let messages = todos
        .into_iter()
        .filter(|it| it.message.len() > 0)
        .map(|it| format!("<li>{}</li>", it.message))
        .collect::<String>();
    format!("{}{}", INDEX, messages)
}

struct Todo {
    message: String,
}

fn main() -> Result<()> {
    let mut todos = vec![];
    println!("Listening to 0.0.0.0:8000");
    let listener = TcpListener::bind("0.0.0.0:8000")?;

    for mut stream in listener.incoming().flatten() {
        // Boiler plate
        let mut buffer = [0; 1024];
        stream.read(&mut buffer)?;
        let mut raw = from_utf8(&buffer)?;
        if let Some(index) = raw.find("\u{0}") {
            raw = &raw[..index];
        }
        let request = raw.split_whitespace().take(2).collect::<String>();

        // Handle paths
        match request.as_str() {
            "GET/" => {
                stream.write(&respond(&index(&todos)))?;
            }
            "POST/create" => {
                if let Some(header) = raw.split("\r\n\r\n").skip(1).next() {
                    if let Some(message) = header.split("=").skip(1).next().map(str::to_string) {
                        todos.push(Todo { message });
                    }
                    stream.write(&respond(&index(&todos)))?;
                }
            }
            unknown => {
                stream.write(&respond(&format!("{:?}\r\nData: {:?}", unknown, raw)))?;
            }
        }
        stream.flush()?;
    }

    Ok(())
}
