#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use rocket::response::content::Html as HtmlResponse;
use rocket::State;
use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;
use rocket_contrib::templates::tera::Context;
use rocket_contrib::templates::tera::Tera;
use scraper::ElementRef;
use scraper::Html;
use scraper::Selector;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use html5ever::tree_builder::TreeSink;
use html5ever::interface::Attribute;
use html5ever::interface::QualName;
use html5ever::tendril::Tendril;
use html5ever::tree_builder::NodeOrText;
use html5ever::tree_builder::ElementFlags;
use markup5ever::LocalName;
use markup5ever::Namespace;


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

#[get("/tramp")]
fn tramp(packages: State<Vec<Package>>) -> HtmlResponse<String> {
    let tramp = packages
        .inner()
        .to_owned()
        .into_iter()
        .find(|p| p.packageurl == "MAG")
        .unwrap()
        .tabs
        .into_iter()
        .find(|t| t.taburl == "Tramp")
        .unwrap()
        .content;
    let mut tera = Tera::default();
    tera.add_template_file("./templates/user.html.tera", Some("user"))
        .unwrap();
    tera.add_raw_template("tramp", &tramp).unwrap();

    let mut skills = HashMap::new();
    skills.insert("12airplane", 50);
    skills.insert("basicbouncing", 100);
    skills.insert("backtuck", 0);

    let mut context = Context::new();
    context.insert("username", "Davide");
    context.insert("userurl", "abcdefg");
    context.insert("skills", &skills);
    HtmlResponse(tera.render("tramp", &context).unwrap())
}

#[get("/pommel")]
fn pommel(packages: State<Vec<Package>>) -> HtmlResponse<String> {
    let pommel = packages
        .inner()
        .to_owned()
        .into_iter()
        .find(|p| p.packageurl == "MAG")
        .unwrap()
        .tabs
        .into_iter()
        .find(|t| t.taburl == "Pommel")
        .unwrap()
        .content;
    let mut tera = Tera::default();
    tera.add_template_file("./templates/user.html.tera", Some("user"))
        .unwrap();
    tera.add_raw_template("pommel", &pommel).unwrap();

    let mut skills = HashMap::new();
    skills.insert("12airplane", 50);
    skills.insert("basicbouncing", 100);
    skills.insert("backtuck", 0);

    let mut context = Context::new();
    context.insert("username", "Davide");
    context.insert("userurl", "abcdefg");
    context.insert("skills", &skills);
    HtmlResponse(tera.render("pommel", &context).unwrap())
}

#[get("/backtuck")]
fn backtuck(skills: State<Vec<Skill>>) -> HtmlResponse<String> {
    let backtuck = skills
        .inner()
        .to_owned()
        .into_iter()
        .find(|s| s.url == "backtuck")
        .unwrap()
        .content;
    let mut tera = Tera::default();
    tera.add_template_file("./templates/docs.html.tera", Some("docs"))
        .unwrap();
    tera.add_raw_template("backtuck", &backtuck).unwrap();
    let mut context = Context::new();
    context.insert("skill", "Back Tuck");
    HtmlResponse(tera.render("backtuck", &context).unwrap())
}

#[derive(Serialize, Clone)]
struct Missing {
    packageurl: String,
    taburl: String,
    skillurl: String,
}

#[get("/missing")]
fn missing(missings: State<Vec<Missing>>) -> HtmlResponse<String> {
    let mut tera = Tera::default();
    tera.add_template_file("./templates/missing.html.tera", Some("missing")).unwrap();
    let mut context = Context::new();
    context.insert("missings", missings.inner());
    HtmlResponse(tera.render("missing", &context).unwrap())
}

fn main() {
    // BUILD NEW FILES AND PARSE TREE FILE

    let mut missings: Vec<Missing> = vec![];

    let skills = load_skills();
    let packages = load_packages(&skills, &mut missings);

    reqwest::blocking::Client::new()
        .post("https://gymskilltree.com/sync")
        .send()
        .ok();

    rocket::ignite()
        .manage(skills)
        .manage(packages)
        .manage(missings)
        .mount("/static", StaticFiles::from("static"))
        .mount(
            "/",
            routes![index, tramp, pommel, backtuck, skills_route, packages_route, missing],
        )
        .launch();
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

        let skill = Skill { url, content };

        skills.push(skill);
    }
    skills
}

fn load_packages(skills: &Vec<Skill>, missings: &mut Vec<Missing>) -> Vec<Package> {
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
                continue;
            }

            let taburl = path.file_stem().unwrap().to_str().unwrap().to_string();
            let content = fs::read_to_string(path).unwrap();
            let content = tabparse(content, &skills, missings, &packageurl, &taburl);

            let tab = Tab { taburl, content };

            tabs.push(tab);
        }

        let package = Package { packageurl, tabs };

        packages.push(package);
    }
    packages
}

#[derive(Serialize)]
struct SkillContext {
    skillinfo: String,
}

fn skillparse(content: String) -> String {
    let mut options = comrak::ComrakOptions::default();
    options.render.unsafe_ = true;
    let template = include_str!("../templates/skill.html");
    template.replace("^@@^", &comrak::markdown_to_html(&content, &options))
}

fn tabparse(content: String, skills: &Vec<Skill>, missings: &mut Vec<Missing>, packageurl: &str, taburl: &str) -> String {
    // goals:
    // parse the skill name
    // parse the skillurl
    // add a three way toggle after text
    // add a .skill class to rect
    // add a color to rect
    
    let mut content = content.replace(r###"fill="#cce5ff""###, "");
    content = content.replace("<br />", "");
    content = content.replace("&#xa;", "");

    let toggle = include_str!("../templates/toggle.html");
    let select_switch = Selector::parse("switch").unwrap();
    let select_foreign_object = Selector::parse("foreignObject").unwrap();
    let select_div = Selector::parse("div").unwrap();
    let select_rect = Selector::parse("rect").unwrap();

    let mut doc = Html::parse_fragment(&content);

    for rect in doc.clone().select(&select_rect) {
            let g = rect
            .next_siblings()
            .find(|n| n.value().is_element())
            .and_then(ElementRef::wrap)
            .unwrap();
        let skilldiv = g
            .select(&select_switch)
            .next()
            .unwrap()
            .select(&select_foreign_object)
            .next()
            .unwrap()
            .select(&select_div)
            .next()
            .unwrap()
            .select(&select_div)
            .next()
            .unwrap()
            .select(&select_div)
            .next()
            .unwrap();
        let skill = skilldiv.clone().inner_html().trim().to_string();
        let skillurl = skill
            .split_whitespace()
            .collect::<String>()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase();

        if !skills.into_iter().map(|x| x.url.clone()).collect::<String>().contains(&skillurl) {
            let missing = Missing {
                packageurl: packageurl.into(),
                taburl: taburl.into(),
                skillurl: skill.clone(),
            };
            missings.push(missing);
        }

        let parent = skilldiv.id();
        //let toggle_selector = Selector::parse("div.tw-toggle").unwrap();
        //let frag = Html::parse_document(&toggle);
        //let toggle = frag.select(&toggle_selector).next().unwrap();
        let skillurlattr = Attribute {
            name: QualName::new(None, Namespace::from(""), LocalName::from("skillurl")),
            value: skillurl.clone().into(),
        };
        let attrs = vec![skillurlattr];
        let flags = ElementFlags::default();
        let toggle = doc.create_element(QualName::new(None, Namespace::from(""), LocalName::from("drag")), attrs, flags);
        let child = NodeOrText::AppendNode(toggle);

        doc.append(&parent, child);

        let class = Attribute {
            name: QualName::new(None, Namespace::from(""), LocalName::from("class")),
            value: Tendril::from("skill"),
        };
        let fill = Attribute {
            name: QualName::new(None, Namespace::from(""), LocalName::from("fill")),
            value: Tendril::from(r###"hsl({% if skills.^@skillurl@^ %}{{ skills.^@skillurl@^ }}{% else %}0{% endif %}, 50%, 50%)"###.replace("^@skillurl@^", &skillurl))
        };
        let attrs = vec![class, fill];
        let target = rect.id();
        doc.add_attrs_if_missing(&target, attrs);
    }

    let mut content = doc.root_element().html(); 
    content = content.replace(r###"<html><!--?xml version="1.0" encoding="UTF-8"?-->"###, "");
    content = content.replace("</html>", "");
    content = content.trim().to_string();
    let select_drag = Selector::parse("drag").unwrap();

    for drag in doc.select(&select_drag) {
        let skill = drag.parent().and_then(ElementRef::wrap).unwrap().text().next().unwrap();
        let skillurl = drag.value().attr("skillurl").unwrap();
        let skilldiv = drag.parent().and_then(ElementRef::wrap).unwrap().html();

        let toggle = toggle.clone().replace("^@skillurl@^", &skillurl).replace("^@skill@^", &skill);

        content = content.replace(&skilldiv, &toggle);
    }


    let template = include_str!("../templates/tab.html");
    template.replace("^@@^", &content)
}
