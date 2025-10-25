use std::{
    collections::HashMap,
    io::{Read, Write},
    net::TcpStream,
};

use tauri::Emitter;

fn send_html(content: &str) -> String {
    let html = format!(
        "<!DOCTYPE html>\n\
         <html>\n\
           <head>\n\
             <meta charset=\"UTF-8\">\n\
             <link rel=\"icon\" href=\"https://waspscripts.dev/favicon.png\">\n\
             <meta name=\"viewport\" content=\"width=device-width\">\n\
             <title>WaspScripts</title>\n\
             <meta name=\"description\" content=\"WaspScripts Simba Login page\">\n\
             <style>\n\
               body {{\n\
                  background-color: #222324;\n\
                  color: white;\n\
                  display: flex;\n\
                  justify-content: center;\n\
                  height: 100vh;\n\
                  text-align: center;\n\
                  flex-direction: column;\n\
               }}\n\
             </style>\n\
           </head>\n\
          <body>\n\
            {content}\n\
          </body>\n\
        </html>"
    );

    let headers = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html\r\n\
         Connection: close\r\n\
         Content-Length: {}\r\n\r\n",
        html.len()
    );

    format!("{headers}{html}")
}

pub fn handle_client(mut stream: TcpStream, app: tauri::AppHandle) -> bool {
    let response = send_html("<h2>Authentication Complete</h2>\r\n        <p>You may now close this window and return to the app.</p>");

    let mut buffer = [0; 1024];
    if stream.read(&mut buffer).is_err() {
        return false;
    }

    let request = String::from_utf8_lossy(&buffer);
    let Some(request_line) = request.lines().next() else {
        return false;
    };

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return false;
    }

    let path = parts[1];
    let Some(q_idx) = path.find('?') else {
        return false;
    };

    let query = &path[q_idx + 1..];
    let params: HashMap<&str, &str> = query
        .split('&')
        .filter_map(|kv| {
            let mut split = kv.splitn(2, '=');
            Some((split.next()?, split.next().unwrap_or("")))
        })
        .collect();

    let code = params.get("code").map(|&s| s.to_string());
    let error = params.get("error").map(|&s| s.to_string());

    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();

    let payload = serde_json::json!({
        "code": code,
        "error": error
    });

    let _ = app
        .emit("oauth-callback", payload)
        .expect("Failed to ping the front-end!");
    true
}
