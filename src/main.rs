use std::{
    io::{Read, Write},
    net::TcpListener,
    str::from_utf8,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn respond(message: String) -> Vec<u8> {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
        message.len(),
        message
    )
    .as_bytes()
    .to_vec()
}

struct Todo {
    message: String,
}

impl Todo {
    fn show(&self, index: usize) -> String {
        format!(
            r#"
<li><form action="/update" method="post">
  <input type="text" name="update" value="{}">
  <input type="hidden" name="index" value="{}">
  <input type="submit" value="Update">
</form></li>
"#,
            self.message, index
        )
    }
}

const INDEX: &str = r#"
<h1>Todos</h1>
<form action="/create" method="post">
  <label for="todo">Add todo note: </label>
  <input type="text" name="todo">
  <input type="submit" value="Submit">
</form>
"#;

fn index(todos: &[Todo]) -> String {
    let messages = todos
        .into_iter()
        .enumerate()
        .map(|(i, it)| it.show(i))
        .collect::<String>();
    format!("{}<ul>{}</ul>", INDEX, messages)
}

fn parse_form(input: &str) -> Option<Vec<&str>> {
    let args = input.split("\r\n\r\n").skip(1).next()?;
    let items = args.split("&").flat_map(|it| it.split("=").skip(1).next()).collect::<Vec<_>>();
    Some(items)
}

fn main() -> Result<()> {
    let mut todos = vec![];
    println!("Listening to 0.0.0.0:8000");
    let listener = TcpListener::bind("0.0.0.0:8000")?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                // Boiler plate
                let mut buffer = [0; 1024];
                stream.read(&mut buffer)?;
                let mut raw = from_utf8(&buffer)?;
                if let Some(index) = raw.find("\u{0}") {
                    raw = &raw[..index];
                }
                let request = raw.split_whitespace().take(2).collect::<String>();

                // Handle requests
                match request.as_str() {
                    "GET/" => {
                        stream.write(&respond(index(&todos)))?;
                    }
                    "POST/create" => {
                        if let Some(args) = parse_form(raw) {
                            if args[0].len() > 0 {
                                todos.push(Todo { message: args[0].to_string() });
                            }
                            stream.write(&respond(index(&todos)))?;
                        }
                    }
                    "POST/update" => {
                        if let Some(args) = parse_form(raw) {
                            let i = args[1].parse::<usize>()?;
                            todos[i].message = args[0].to_string();
                            stream.write(&respond(index(&todos)))?;
                        }
                    }
                    unknown => {
                        stream.write(&respond(format!("{:?}\r\nData: {:?}", unknown, raw)))?;
                    }
                }
                stream.flush()?;
            }
            Err(e) => println!("Error occured: {:?}", e),
        }
    }
    Ok(())
}
