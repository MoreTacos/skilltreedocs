#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use std::collections::HashMap;
use rocket::State;
use std::fs;
use serde::Serialize;
use rocket_contrib::json::Json;

#[get("/skill?<s>")]
fn skill(skills: State<HashMap<String, String>>, s: String) -> String {
    skills.get(s.as_str()).unwrap().to_string()
}

#[get("/package?<p>&<t>")]
fn package(packages: State<HashMap<String, HashMap<String, String>>>, t: String, p: String) -> String {
    packages.get(p.as_str()).unwrap().get(t.as_str()).unwrap().to_string()
}

#[derive(Serialize)]
struct Tabs {
    tabs: Vec<String>
}

#[get("/tabs?<p>")]
fn tabs(packages: State<HashMap<String, HashMap<String, String>>>, p: String) -> Json<Vec<String>> {
    let package: Vec<String> = packages.get(p.as_str()).unwrap().keys().cloned().collect();
    Json(package)
}

fn main() {
    // BUILD NEW FILES AND PARSE TREE FILE
    let skills = load_skills();
    let packages = load_packages();

    rocket::ignite().manage(skills).manage(packages).mount("/", routes![skill, package, tabs]).launch();
}

fn load_skills() -> HashMap<String, String> {
    let mut skills = HashMap::new();
    for skill in fs::read_dir("./pages").unwrap() {
        let path = skill.unwrap().path();

        if !path.to_str().unwrap().contains(".md") {
            continue;
        }

        let skill = path.file_stem().unwrap().to_str().unwrap().to_string();
        let content = fs::read_to_string(path).unwrap();

        skills.insert(skill, content);
    }
    skills
}

fn load_packages() -> HashMap<String, HashMap<String, String>> {
    let mut packages = HashMap::new();
    for package in fs::read_dir("./packages").unwrap() {
        let path = package.unwrap().path();

        if !path.is_dir() {
            continue;
        }
        let packagename = path.file_stem().unwrap().to_str().unwrap().to_string();

        let mut package = HashMap::new();
        for tab in fs::read_dir(path).unwrap() {
            let path = tab.unwrap().path();

            if !path.to_str().unwrap().contains(".svg") {
                continue
            }

            let tabname = path.file_stem().unwrap().to_str().unwrap().to_string();
            let content = fs::read_to_string(path).unwrap();

            package.insert(tabname, content);
        }

        packages.insert(packagename, package);
    }
    packages
}
