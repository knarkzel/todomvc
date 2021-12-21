use std::str::from_utf8;
use std::{
    io::{Read, Write},
    net::TcpListener,
};
use std::collections::HashMap;

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

const INDEX: &str = r#"
<h1>Todos</h1>
<form action="/create" method="post">
  <label for="todo">Add todo note:</label>
  <input type="text" id="todo" name="todo">
  <input type="submit" value="Submit">
</form>
"#;

struct Note {
    message: String,
}

fn main() -> Result<()> {
    println!("Listening to 0.0.0.0:8000");
    let listener = TcpListener::bind("0.0.0.0:8000")?;
    let mut id = 0;
    let mut notes: HashMap<usize, Note> = HashMap::new();

    for stream in listener.incoming() {
        // Boiler plate
        let mut stream = stream?;
        let mut buffer = [0; 1024];
        stream.read(&mut buffer)?;
        let data = from_utf8(&buffer)?;
        let input = data.split_whitespace().take(2).collect::<Vec<_>>();
        let paths = input[1]
            .split("/")
            .filter(|path| path.len() > 0)
            .collect::<Vec<_>>();

        // Handle paths
        match (input[0], &paths[..]) {
            ("GET", []) => {
                let messages = notes.values().map(|it| {
                    format!("<li>{}</li>", it.message)
                }).collect::<Vec<_>>();
                let index = format!("{}{}", INDEX, messages.join(""));
                stream.write(&respond(&index))?;
            }
            ("POST", ["create"]) => {
                if let Some(Some(header)) = data
                    .split("\r\n\r\n")
                    .skip(1)
                    .next()
                    .map(|it| it.split('\u{0}').next())
                {
                    let args = header.split("&").map(|it| {
                        let items = it.split("=").collect::<Vec<_>>();
                        (items[0], items[1])
                    }).collect::<HashMap<_, _>>();
                    let note = Note { message: args["todo"].to_string() };
                    notes.insert(id, note);
                    id += 1;
                    let messages = notes.values().map(|it| {
                        format!("<li>{}</li>", it.message)
                    }).collect::<Vec<_>>();
                    let index = format!("{}{}", INDEX, messages.join(""));
                    stream.write(&respond(&index))?;
                }
            }
            (method, paths) => {
                stream.write(&respond(&format!(
                    "Method: {:?}\r\nPaths: {:?}\r\nData: {:?}",
                    method, paths, data
                )))?;
            }
        }
        stream.flush()?;
    }

    Ok(())
}
