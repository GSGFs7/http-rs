use std::path::Path;

use crate::{body::HttpBody, request::HttpRequest, response::HttpResponse};

use tokio::{fs::File, io::AsyncReadExt};

// TODO: Configurable
pub async fn file_server_handler(req: HttpRequest) -> HttpResponse {
    let path = format!("./www{}", req.uri.as_string());

    println!("File server request for: {path}");

    if !Path::new(&path).exists() {
        return HttpResponse::new(404, "Not Found").with_body("Not found".into());
    }

    let file = File::open(path).await;
    match file {
        Ok(mut file) => {
            let metadata = match file.metadata().await {
                Ok(meta) => meta,
                Err(e) => {
                    eprintln!("Failed to get metadata: {e}");
                    return HttpResponse::new(500, "Internal Server Error")
                        .with_body("Error reading file metadata".into());
                }
            };

            let file_size = metadata.len() as usize;

            if file_size == 0 {
                return HttpResponse::new(204, "No Content");
            }

            if file_size < 1024 * 1024 {
                let mut data = Vec::with_capacity(file_size);
                match file.read_to_end(&mut data).await {
                    Ok(data) => data,
                    Err(e) => {
                        eprintln!("Failed to read file: {e}");
                        return HttpResponse::new(500, "Internal Server Error")
                            .with_body("Error reading file".into());
                    }
                };

                HttpResponse::new(200, "OK")
                    .with_body(HttpBody::from(data))
                    .insert_header("Content-Length", &file_size.to_string())
                    .insert_header("Content-Type", "application/octet-stream")
                    .insert_header("Cache-Control", "public, max-age=31536000")
            } else {
                HttpResponse::new(200, "OK")
                    .with_streaming_body(file, 8192)
                    .insert_header("Content-Length", &file_size.to_string())
                    .insert_header("Content-Type", "application/octet-stream")
                    .insert_header("Accept-Ranges", "bytes")
                    .insert_header("Cache-Control", "public, max-age=31536000")
            }
        }
        Err(e) => {
            eprintln!("Failed to open file: {e}");
            HttpResponse::new(404, "Not Found").with_body("Not found".into())
        }
    }
}
