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

#[get("/list_packages")]
fn list_packages(packages: State<HashMap<String, HashMap<String, String>>>) -> Json<Vec<String>> {
    let keys: Vec<String> = packages.keys().cloned().collect();
    Json(keys)
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

    rocket::ignite().manage(skills).manage(packages).mount("/", routes![skill, package, tabs, list_packages]).launch();
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
            let content = tabparse(content);

            package.insert(tabname, content);
        }

        packages.insert(packagename, package);
    }
    packages
}

fn tabparse(content: String) -> String {
    let mut svg = content;
    // Remove all <span> tags
    svg = svg.replace(r"<span>", "");
    svg = svg.replace(r"</span>", "");

    let mut sliced = svg.split(r"<rect");

    // Removing the first slice, which is irrelevant

    let mut svg = r###"{% extends "user" %}
{% block tree %}
"###
    .to_string()
        + sliced.next().unwrap();

    let sliced: Vec<_> = sliced.collect();

    for slice in sliced {
        let mut slice = slice.to_string();
        if slice.contains("span") {
            println!("Element containing span might not be displayed");
        }

        // find skill
        let mut search_domain = slice.to_string().clone();

        // closer to answer 1
        let from = search_domain.find("word-wrap").unwrap();
        search_domain = search_domain[from..].to_string();

        let from2 = search_domain.find(">").unwrap();
        let to = search_domain.find("<").unwrap();

        let skill_exact = search_domain[from2 + 1..to].to_string();

        let skill = skill_exact
            .split_whitespace()
            .collect::<String>()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase();

        let skill_exact_correct = slice.to_string().clone()[from..from + to].to_string();

        // skip if empty
        if skill == "".to_string() {
            println!("Skipped empty box");
            continue;
        }

        let color = "{% if skills.".to_string()
            + &skill
            + "%}{{ skills."
            + &skill
            + "}}{% else %}0{% endif %}";

        // input slider
        let onchange = format!(
            r###"fetch(`/update?u={{{{ userhash }}}}&s={}&v=${{this.value}}`, {{ method: 'PUT' }})"###,
            &skill
        );
        let oninput = r###"this.closest('g').previousElementSibling.style.fill = `hsl(${this.value}, 50%, 50%)`"###;
        let mut skill_exact_correct_with_input = skill_exact_correct.clone()
            + r###"<input type="range" min="0" max="100" value=""###
            + &color
            + r###"" onchange=""###
            + &onchange
            + r###"" oninput=""###
            + &oninput
            + r###"">"###;
        skill_exact_correct_with_input = skill_exact_correct_with_input.replace(
            &skill_exact,
            &format!(
                r###"<p><a href="/skill?s={}">{}</a></p>"###,
                &skill, &skill_exact
            ),
        );
        slice = slice.replace(&skill_exact_correct, &skill_exact_correct_with_input);

        // Skill value finder and remove (A) | (B) | (C) etc...

        let mut skillvalue: String = "".to_string();

        for c in "ABCDEFGHIabcdefghi".chars() {
            let search = format!("({})", &c);
            if slice.contains(&search) {
                skillvalue = c.to_string();
                slice = slice.replace(&search, "");
            }
        }

        svg =
            svg + r###"<rect fill="hsl("### + &color + r###", 50%, 50%)" class="skill""### + &slice;
    }

    svg = svg
        + r###"
{% endblock %}"###;

    svg = svg.replace(r"<br>", "");
    svg.to_string()
}
