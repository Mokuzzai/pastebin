
use super::schema::post;

#[derive(Insertable)]
#[table_name="post"]
pub struct NewPost<'a> {
	pub post_id: &'a str,
	pub file_id: &'a str,
}
