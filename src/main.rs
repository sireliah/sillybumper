extern crate hyper;

use clap::{App, Arg};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;

use hyper::rt::Future;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};

fn parse_contents(contents: &str) -> Result<(i32, i32), Box<std::error::Error>> {
    println!("{:?}", contents);
    let result: Result<Vec<i32>, _> = contents
        .trim()
        .split('/')
        .collect::<Vec<&str>>()
        .iter()
        .map(|&element| element.parse::<i32>())
        .collect();

    match result {
        Ok(vec) => match vec.len() {
            2 => Ok((vec[0], vec[1])),
            _ => {
                let err = "Incorrect number of digits in file";
                eprintln!("{}", err);
                Err(From::from(err.to_string()))
            }
        },
        Err(err) => Err(Box::new(err)),
    }
}

fn read_params() -> (u16, String) {
    let matches = App::new("Silly Bumper")
        .version("0.1.0")
        .author("Psot")
        .about("Listens for requests and bumps int in file. Kochamkota. 😻")
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .takes_value(true)
                .help("Port to run server on. Defaults to 4000."),
        )
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .takes_value(true)
                .help("Which file to read/write. Defaults to file.txt"),
        )
        .get_matches();

    let file = matches.value_of("file").unwrap_or("file.txt");
    let port: u16 = match matches.value_of("port") {
        Some(value) => match value.parse::<u16>() {
            Ok(v) => v,
            Err(_) => {
                eprintln!("Could not parse port, using default");
                4000
            }
        },
        None => 4000,
    };
    (port, file.to_string())
}

fn read_file(file_path: &Path) -> Result<String, Box<std::error::Error>> {
    let mut file = File::open(&file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn write_file(file_path: &Path, l_value: i32, r_value: i32) -> Result<(), Box<std::error::Error>> {
    let mut file = File::create(&file_path)?;
    Ok(write!(file, "{}/{}", l_value, r_value)?)
}

fn call(req: Request<Body>, file_path: Arc<String>) -> Result<Response<Body>, hyper::Error> {
    let p = Arc::try_unwrap(file_path).unwrap();
    let path = Path::new(&p);

    println!("{:?}", req);
    let value = match read_file(&path) {
        Ok(value) => value,
        Err(_err) => "No data".to_string(),
    };
    let (left, _right) = match parse_contents(&value) {
        Ok((l, r)) => match write_file(&path, l + 1, r) {
            Ok(_) => (l, r),
            Err(err) => {
                println!("{:?}", err);
                return Ok(Response::builder()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .body(Body::from("Failed to write a file".to_string()))
                    .unwrap());
            }
        },
        Err(_err) => {
            return Ok(Response::builder()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .body(Body::from("Cannot parse config".to_string()))
                .unwrap())
        }
    };
    Ok(Response::new(Body::from(left.to_string())))
}

fn main() {
    let (port, file_path) = read_params();

    let addr = ([127, 0, 0, 1], port).into();
    let path_to_print = file_path.clone();
    let path = Arc::new(file_path);

    let service = make_service_fn(move |_| {
        let path_clone = path.clone();
        service_fn(move |req: Request<Body>| {
            let inner = path_clone.clone();
            call(req, inner)
        })
    });

    let server = Server::bind(&addr)
        .serve(service)
        .map_err(|e| eprintln!("server error: {:?}", e));

    println!("Listening on {:?}, for file {}.", addr, path_to_print);
    hyper::rt::run(server);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_contents() {
        let result = parse_contents("10/30").unwrap();
        assert_eq!(result, (10, 30));
    }
}
