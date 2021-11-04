
#![feature(decl_macro)]
#![feature(proc_macro_hygiene)]

use rocket::*;
use rocket::http::RawStr;
use rocket::request::Form;
use rocket::response::content::Html;

use std::fs::File;
use std::io;

#[get("/")]
fn index() -> Html<io::Result<File>> {
	Html(File::open("assets/index.html"))
}

#[get("/<id>")]
fn retrieve(id: u32) -> Result<(), ()> {
	todo!()
}

#[derive(FromForm)]
struct Upload<'a> {
	paste: &'a RawStr,
	author: Option<&'a RawStr>,
}

#[post("/", data = "<upload>")]
fn upload(upload: Form<Upload>) -> &'static str {
	"uploaded"
}

fn main() {
    rocket::ignite()
		.mount("/", routes![index, upload])
		.launch();
}
