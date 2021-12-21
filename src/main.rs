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
    marked: bool,
}

impl Todo {
    fn show(&self, index: usize) -> String {
        let id = if self.marked { "marked" } else { "not-marked" };
        format!(
            r#"
<li>
<form action="/mark" method="post">
    <input type="hidden" name="index" value="{}">
    <input type="submit" id="mark" value="Mark">
</form>
<form action="/update" method="post">
  <input type="text" class="{}" name="update" value="{}" autocomplete="off">
  <input type="hidden" name="index" value="{}">
</form>
<form action="/delete" method="post">
  <input type="hidden" name="index" value="{}">
  <input type="submit" id="delete" value="Delete">
</form>
</li>
"#,
            index, id, self.message, index, index
        )
    }
}

const INDEX: &str = r#"
<style>
body { max-width: 800px; margin: 0 auto; font-size: 28px; font-family: "Helvetica", "Arial", sans-serif; }
a { color: #e81c4f; }
h1 { text-align: center; margin: 0; }
form { display: inline; }
ul { margin: 10px 0; padding: 0; }
li { list-style-type: none; position: relative; margin: 10px 0; }
input[type=text] { font-size: 19px; padding: 0.5em; width: 100%; }
input[type=submit] { font-size: 20px; height: 45px; position: absolute; }
#delete { right: 0px; }
#mark { right: 73px; }
.marked { color: grey; text-decoration: line-through; }
p { margin: 0; }
</style>
<h1>todos</h1>
<form action="/create" method="post">
  <input type="text" name="todo" placeholder="What needs to be done?" autocomplete="off">
</form>
"#;

enum Filter {
    All,
    Active,
    Completed,
}

fn menu(todos: &[Todo]) -> String {
    let left = todos.into_iter().filter(|it| !it.marked).count();
    if todos.len() > 0 {
        format!(
            r#"
                <p>
                Items left: {}
                <a href="\#">All</a>, 
                <a href="/active">Active</a>,
                <a href="/completed">Completed</a>
                </p>
                "#,
            left
        )
    } else {
        String::new()
    }
}

fn index(todos: &[Todo], filter: Filter) -> String {
    let messages = match filter {
        Filter::All => todos
            .into_iter()
            .enumerate()
            .map(|(i, it)| it.show(i))
            .collect::<String>(),
        Filter::Active => todos
            .into_iter()
            .filter(|it| !it.marked)
            .enumerate()
            .map(|(i, it)| it.show(i))
            .collect::<String>(),
        Filter::Completed => todos
            .into_iter()
            .filter(|it| it.marked)
            .enumerate()
            .map(|(i, it)| it.show(i))
            .collect::<String>(),
    };
    format!("{}{}<ul>{}</ul>", INDEX, messages, menu(todos))
}

fn parse_form(input: &str) -> Option<Vec<&str>> {
    let args = input.split("\r\n\r\n").skip(1).next()?;
    let items = args
        .split("&")
        .flat_map(|it| it.split("=").skip(1).next())
        .collect::<Vec<_>>();
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
                        stream.write(&respond(index(&todos, Filter::All)))?;
                    }
                    "POST/create" => {
                        if let Some(args) = parse_form(raw) {
                            if args[0].len() > 0 {
                                todos.push(Todo {
                                    message: args[0].to_string(),
                                    marked: false,
                                });
                            }
                            stream.write(&respond(index(&todos, Filter::All)))?;
                        }
                    }
                    "POST/update" => {
                        if let Some(args) = parse_form(raw) {
                            let i = args[1].parse::<usize>()?;
                            if let Some(mut todo) = todos.get_mut(i) {
                                todo.message = args[0].to_string();
                            }
                            stream.write(&respond(index(&todos, Filter::All)))?;
                        }
                    }
                    "POST/delete" => {
                        if let Some(args) = parse_form(raw) {
                            let i = args[0].parse::<usize>()?;
                            if i < todos.len() {
                                todos.remove(i);
                            }
                            stream.write(&respond(index(&todos, Filter::All)))?;
                        }
                    }
                    "POST/mark" => {
                        if let Some(args) = parse_form(raw) {
                            let i = args[0].parse::<usize>()?;
                            if i < todos.len() {
                                todos[i].marked = !todos[i].marked;
                            }
                            stream.write(&respond(index(&todos, Filter::All)))?;
                        }
                    }
                    "GET/active" => {
                        stream.write(&respond(index(&todos, Filter::Active)))?;
                    }
                    "GET/completed" => {
                        stream.write(&respond(index(&todos, Filter::Completed)))?;
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
