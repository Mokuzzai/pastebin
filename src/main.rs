
#![feature(decl_macro)]
#![feature(proc_macro_hygiene)]

mod schema;
mod models;

#[macro_use] extern crate diesel;

use std::fs::File;
use std::sync::Mutex;

use rocket::*;
use rocket::response::Redirect;
use rocket::response::content::Html;

use diesel::prelude::*;

use anyhow::Result;

use uuid::Uuid;

#[get("/")]
fn index() -> Html<Result<File>> {
	Html(File::open("assets/index.html").map_err(Into::into))
}

struct App {
	connection: SqliteConnection,
}

impl App {
	fn new(path: &str) -> Result<Self> {
		let connection = SqliteConnection::establish(path)?;

		Ok(Self { connection })
	}
	fn call_with_uuid<B>(uuid: &Uuid, mut f: impl FnMut(&str) -> B) -> B {
		let mut buffer = Uuid::encode_buffer();

		f(uuid.to_hyphenated().encode_lower(&mut buffer))
	}
	fn uploads_dir(file_id: &Uuid) -> String {
		Self::call_with_uuid(file_id, |file_id| format!("uploads/{}", file_id))
	}
	fn upload(&self, data: Data) -> Result<Uuid> {
		let post_id = Uuid::new_v4();
		let file_id = Uuid::new_v4();

		let path = Self::uploads_dir(&file_id);

		data.stream_to_file(path)?;

		let _ = Self::call_with_uuid(&post_id, |post_id| {
			Self::call_with_uuid(&file_id, |file_id| {
				let new_post = models::NewPost { post_id, file_id };

				diesel::insert_into(schema::post::table)
					.values(&new_post)
					.execute(&self.connection)
			})
		})?;

		Ok(post_id)
	}
	fn retrieve(&self, post_id: Uuid) -> Result<File> {
		Self::call_with_uuid(&post_id, |post_id| {
			let file_id = schema::post::table
				.select(schema::post::file_id)
				.filter(schema::post::post_id.eq(post_id))
				.get_result::<String>(&self.connection)?;

			let file_id = Uuid::parse_str(&file_id)?;

			let path = Self::uploads_dir(&file_id);

			File::open(path).map_err(Into::into)
		})
	}
}

#[post("/", data = "<paste>")]
fn upload(db: State<Mutex<App>>, paste: Data) -> Result<Redirect> {
	println!("{}", String::from_utf8(paste.peek().to_vec())?);

	let lock = db.lock().unwrap();

	let mut buffer = Uuid::encode_buffer();

	let post_id = {
		let post_id = lock.upload(paste)?;

		post_id
			.to_hyphenated()
			.encode_lower(&mut buffer)
	};

	Ok(Redirect::to(uri!(retrieve: &*post_id)))
}

#[get("/post/<post_id>")]
fn retrieve(db: State<Mutex<App>>, post_id: String) -> Result<File> {
	let post_id = Uuid::parse_str(&post_id)?;

	let lock = db.lock().unwrap();

	lock.retrieve(post_id)
}

fn main() -> Result<()> {
    rocket::ignite()
		.manage(Mutex::new(App::new("posts.db")?))
		.mount("/", routes![index, upload, retrieve])
		.launch();

	Ok(())
}
