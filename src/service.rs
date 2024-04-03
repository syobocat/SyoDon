pub mod post;
pub mod sign_header;

#[derive(Clone, Copy)]
pub enum Method {
    Get,
    Post,
}
