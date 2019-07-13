extern crate hyper;

use std::env;
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write};

use hyper::{Body, Response, Request, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use hyper::rt::Future;

static PORT: u16 = 4000;


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
        Ok(vec) => {
            match vec.len() {
                2 => Ok((vec[0], vec[1])),
                _ => {
                    let err = "Incorrect number of digits in file";
                    eprintln!("{}", err);
                    Err(From::from(err.to_string()))
                }
            }
        },
        Err(err) => Err(Box::new(err)),
    }
}

fn read_params() -> Option<String> {
    let args: Vec<String> = env::args().collect();
    args.into_iter().nth(1)
}

fn read_file(file_path: &str) -> Result<String, Box<std::error::Error>> {
    let path = Path::new(file_path);
    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

fn write_file(file_path: &str, l_value: i32, r_value: i32) -> Result<(), Box<std::error::Error>> {
    let path = Path::new(file_path);
    let mut file = File::create(&path)?;
    Ok(write!(file, "{}/{}", l_value, r_value)?)
}

fn call(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let file_path = match read_params() {
        Some(value) => value,
        None => "file.txt".to_string()
    };
    println!("{:?}", req);
    let value = match read_file(&file_path) {
        Ok(value) => value,
        Err(_err) => "No data".to_string(),
    };
    let (left, _right) = match parse_contents(&value) {
        Ok((l, r)) => {
            match write_file(&file_path, l + 1, r) {
                Ok(_) => (l, r),
                Err(err) => {
                    println!("{:?}", err);
                    return Ok(Response::builder()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .body(Body::from("Failed to write a file".to_string()))
                    .unwrap())
                }
            }
        },
        Err(_err) => return Ok(Response::builder()
               .status(StatusCode::UNPROCESSABLE_ENTITY)
               .body(Body::from("Cannot parse config".to_string()))
               .unwrap())
    };
    Ok(Response::new(Body::from(left.to_string())))
}

fn main() {
    let addr = ([127, 0, 0, 1], PORT).into();

    let service = make_service_fn(|_| service_fn(call));

    let server = Server::bind(&addr)
        .serve(service).map_err(|e| eprintln!("server error: {:?}", e));

    println!("Listening on {:?}", addr);
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