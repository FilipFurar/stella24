use crate::AppStella;
use askama::Template;
use std::fs;
// bring trait in scope

#[derive(Template)] // this will generate the code...
#[template(path = "schema.html")] // using the template in this path, relative
// to the `templates` dir in the crate root
struct HelloTemplate<'a> {
    // the name of the struct can be anything
    name: &'a str, // the field name should match the variable name
                   // in your template
}

impl AppStella {
    pub fn to_html(&self, path: &str) {
        let hello = HelloTemplate { name: "world" }; // instantiate your struct
        fs::write(path, hello.name).expect("TODO: panic message");
    }
}
