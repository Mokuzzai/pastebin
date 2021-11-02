#![feature(str_split_whitespace_as_str)]

use std::fs;
use std::net;

use std::io;
use std::io::prelude::*;

type Result<T, E = io::Error> = std::result::Result<T, E>;

struct State {
	listener: net::TcpListener,
}

#[derive(Debug)]
struct HttpRequest<'a> {
	method: &'a str,
	request_uri: &'a str,
	http_version: &'a str,
	headers: &'a str,
	message_body: &'a str,
}

#[derive(Debug)]
enum ParseHttpRequestError {
	MissingMethod,
	MissingRequestUri,
	MissingHttpVersion,
	MissingHeaders,
	MissingMessageBody,
}

impl<'a> HttpRequest<'a> {
	fn parse(src: &'a str) -> Result<Self, ParseHttpRequestError> {
		let mut parts = src.split_ascii_whitespace();

		let method = parts.next().ok_or(ParseHttpRequestError::MissingMethod)?;
		let request_uri = parts
			.next()
			.ok_or(ParseHttpRequestError::MissingRequestUri)?;

		let mut rest = parts.as_str().split("\r\n");

		let http_version = rest
			.next()
			.ok_or(ParseHttpRequestError::MissingHttpVersion)?;


		let headers = rest.next().ok_or(ParseHttpRequestError::MissingHeaders)?;
		let message_body = rest
			.next()
			.ok_or(ParseHttpRequestError::MissingMessageBody)?;

		Ok(Self {
			method,
			request_uri,
			http_version,
			headers,
			message_body,
		})
	}
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
				println!("begin");

				let mut stream = result.unwrap();

				let mut body = {
					let mut buffer = [0; 1024];

					let bytes_read = stream.read(&mut buffer).unwrap();

					let bytes = buffer.get(0..bytes_read).unwrap();

					String::from_utf8(bytes.to_owned()).unwrap()
				};

				let http_request = HttpRequest::parse(&body).unwrap();

				println!("got `{:#?}`", http_request);

				let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", body.len(), body);

				stream.write(response.as_bytes()).unwrap();
				stream.flush().unwrap();

				println!("sent `{}`", response)

			} else {
				// `TcpListener::incoming` will never return `None`
				unreachable!()
			}
		}
	}
}

fn handle_connection(mut stream: net::TcpStream) -> Result<()> {
	let mut buffer = [0; 1024];

	stream.read(&mut buffer)?;

	let response = format!(
		"HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
		5, "Hello",
	);

	stream.write(response.as_bytes())?;
	stream.flush()?;

	Ok(())
}

fn main() -> Result<()> {
	let state = State::new()?;

	state.run()
}
