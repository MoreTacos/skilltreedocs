#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use serde::Serialize;
use rocket::State;
use std::fs;
use rocket_contrib::json::Json;

#[derive(Serialize, Clone)]
struct Package {
    packageurl: String,
    tabs: Vec<Tab>,
}

#[derive(Serialize, Clone)]
struct Tab {
    taburl: String,
    content: String,
}

#[derive(Serialize, Clone)]
struct Skill {
    url: String,
    content: String,
}

#[get("/")]
fn index() -> String {
    "Welcome to skilltreedocs. Try to /skills and /packages route for relevant paths".to_string()
}

#[get("/skills")]
fn skills_route(skills: State<Vec<Skill>>) -> Json<Vec<Skill>> {
    let skills = skills.inner().to_owned();
    Json(skills)
}

#[get("/packages")]
fn packages_route(packages: State<Vec<Package>>) -> Json<Vec<Package>> {
    let packages = packages.inner().to_owned();
    Json(packages)
}

fn main() {
    // BUILD NEW FILES AND PARSE TREE FILE
    let skills = load_skills();
    let packages = load_packages();

    reqwest::blocking::Client::new().post("https://gymskilltree.com/sync").send().ok();

    rocket::ignite().manage(skills).manage(packages).mount("/", routes![index, skills_route, packages_route]).launch();
}

fn load_skills() -> Vec<Skill> {
    let mut skills = vec![];
    for skill in fs::read_dir("./pages").unwrap() {
        let path = skill.unwrap().path();

        if !path.to_str().unwrap().contains(".md") {
            continue;
        }

        let url = path.file_stem().unwrap().to_str().unwrap().to_string();
        let content = fs::read_to_string(path).unwrap();
        let content = skillparse(content);

        let skill = Skill {
            url,
            content,
        };

        skills.push(skill);
    }
    skills
}

fn load_packages() -> Vec<Package> {
    let mut packages = vec![];
    for package in fs::read_dir("./packages").unwrap() {
        let path = package.unwrap().path();

        if !path.is_dir() {
            continue;
        }
        let packageurl = path.file_stem().unwrap().to_str().unwrap().to_string();

        let mut tabs = vec![];
        for tab in fs::read_dir(path).unwrap() {
            let path = tab.unwrap().path();

            if !path.to_str().unwrap().contains(".svg") {
                continue
            }

            let taburl = path.file_stem().unwrap().to_str().unwrap().to_string();
            let content = fs::read_to_string(path).unwrap();
            let content = tabparse(content);

            let tab = Tab {
                taburl,
                content,
            };

            tabs.push(tab);
        }

        let package = Package {
            packageurl,
            tabs,
        };

        packages.push(package);
    }
    packages
}

fn skillparse(content: String) -> String {
    let mut options = comrak::ComrakOptions::default();
    options.render.unsafe_ = true;
    let content = comrak::markdown_to_html(&content, &options);
    let content = r###"{% extends "docs" %}

{% block body %}"###.to_string() + &content + r###"
<div class="issue"><a href="https://github.com/MoreTacos/skilltreedocs/tree/master/pages/{{ skill }}.md">Add something to the page?</a></div>"### + r###"
{% endblock %}"###;
    content.to_string()
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
