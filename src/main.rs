#![feature(str_split_whitespace_as_str)]

use std::fs;
use std::net;

use std::io;
use std::io::ErrorKind;
use std::io::prelude::*;

type Result<T, E = io::Error> = std::result::Result<T, E>;

// GET /
fn index() -> &'static str {
	"
	USAGE

		POST /

			accepts raw data in the body of the request and responds with a URL of
			a page containing the body's content

		GET /<id>

			retrieves the content for the paste with id `<id>`
	"
}

// POST /
fn upload(data: &str) -> Result<String> {
	use std::hash::*;
	use std::collections::hash_map::DefaultHasher;

	let mut hasher = DefaultHasher::new();

	data.hash(&mut hasher);

	let id = hasher.finish();

	let path = format!("upload/{}", id);

	fs::write(path, &*data)?;

	Ok(id.to_string())
}

// GET /<id>
fn retrieve(id: u64) -> Result<String> {
	fs::read_to_string(format!("upload/{}", id))
}

#[derive(Debug)]
struct HttpRequest<'a> {
	method: Method,
	request_uri: &'a str,
	http_version: &'a str,
	headers: &'a str,
	message_body: &'a str,
}

#[derive(Debug)]
enum Method {
	POST,
	GET,
}

impl<'a> HttpRequest<'a> {
	fn parse(src: &'a str) -> Option<Self> {
		let mut parts = src.split_ascii_whitespace();

		let method = parts.next().map(|method| match method {
			"GET" => Some(Method::GET),
			"POST" => Some(Method::POST),
			_ => None,
		})??;

		let request_uri = parts.next()?;

		let mut rest = parts.as_str().split("\r\n");

		let http_version = rest.next()?;

		let headers = rest.next()?;
		let message_body = rest.next()?;

		Some(Self {
			method,
			request_uri,
			http_version,
			headers,
			message_body,
		})
	}
}

struct State {
	listener: net::TcpListener,
}

impl State {
	fn new() -> Result<Self> {
		let addr = net::SocketAddrV4::new(net::Ipv4Addr::LOCALHOST, 8080);

		let listener = net::TcpListener::bind(addr)?;

		Ok(Self { listener })
	}

	fn run(mut self) -> ! {
		let mut iter = self.listener.incoming();

		loop {
			if let Some(result) = iter.next() {
				if let Err(error) = handle_connection(result) {
					eprintln!("{}", error);
				}
			} else {
				// `TcpListener::incoming` will never return `None`
				unreachable!()
			}
		}
	}
}

fn handle_connection(stream: Result<net::TcpStream>) -> Result<()> {
	let mut stream = stream?;
	let mut buffer = [0; 1024];
	let mut body = {
		let bytes_read = stream.read(&mut buffer)?;

		let bytes = buffer.get(0..bytes_read).ok_or(ErrorKind::Other)?;

		String::from_utf8_lossy(bytes)
	};

	let http_request = HttpRequest::parse(body.as_ref()).ok_or(ErrorKind::Other)?;

	handle_http_request(stream, http_request)
}

fn handle_http_request(mut stream: net::TcpStream, hr: HttpRequest) -> Result<()> {
	eprintln!("{:#?}", hr);

	match hr.method {
		Method::GET => {
			match hr.request_uri {
				"/" => {
					let resp = index();

					stream.write(resp.as_bytes())?;
					stream.flush()?;
				},
				id => {
					let id = id[1..].parse().unwrap();

					let resp = retrieve(id)?;

					stream.write(resp.as_bytes())?;
					stream.flush()?;
				},
			}
		},
		Method::POST => {
			let resp = upload(hr.message_body)?;

			stream.write(resp.as_bytes())?;
			stream.flush()?;
		}
	}

	Ok(())
}

fn main() -> Result<()> {
	State::new()?.run();
}
