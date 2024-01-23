//! A web server to proxy the files stored by users. File Garden exposes this through
//! `https://file.garden/...`.
//!
//! For security, this server must be exposed on a separate origin from the website. Otherwise, a
//! user could upload an HTML file containing an XSS attack (for example, sending a request to the
//! website's API authenticated by the client's cookies, allowing a page to act on behalf of a user
//! without their knowledge).

fn main() {}
