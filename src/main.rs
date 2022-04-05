use std::{collections::HashMap, io::{Read, Write}};

use regex::Captures;

fn main () {
    main_inner(std::env::args());
}

pub fn main_inner(mut args: impl Iterator<Item = String>) {
    let mut vars = HashMap::new();
    // get all the env variables into a hashmap
    for (key, value) in std::env::vars() {
        vars.insert(key, value);
    }

    // ignore the executable arg
    let _ = args.next().unwrap();

    let folder = args.next().unwrap();

    // split each arg by = and put it into a hashmap
    for arg in args {
        let mut split = arg.splitn(2, "=");
        let key = split.next().unwrap();
        let value = split.next().unwrap();
        vars.insert(key.to_string(), value.to_string());
    }

    // resolve the current working directory from where this command is run
    let cwd = std::env::current_dir().unwrap();

    // resolve the folder relative to the current working directory
    let folder = cwd.join(folder);

    // read all the files ending with yml in the folder and put them into a hashmap
    let mut files = HashMap::new();
    for entry in std::fs::read_dir(folder).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().unwrap() == "yml" {
            let mut file = std::fs::File::open(path.clone()).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            files.insert(path, contents);
        }
    }
    // replace ${VAR} with the value of the variable across all the files

    let var = regex::Regex::new(r"\$\{(?P<name>[^}]*)\}").unwrap();
    for (_, contents) in &mut files {
        let str = &**contents;
        *contents = var.replace_all(str, |caps: &Captures| {
            let cap_name = caps.name("name").unwrap().as_str().to_string();
            vars.get(&cap_name).expect(format!("{} was not found in environment variables or args", cap_name).as_str())
        }).to_string()
    }

    // get the temporary directory
    let tmp = std::env::temp_dir();

    // write all the files to the temporary directory inside of env_subst folder
    // create the folder if it does not exist

    let folder = tmp.join("env_subst");
    std::fs::create_dir_all(&folder).unwrap();

    // clear the folder
    for entry in std::fs::read_dir(folder.clone()).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        std::fs::remove_file(path).unwrap();
    }

    // create the files
    for (path, contents) in &files {
        let path = folder.join(path.file_name().unwrap());
        let mut file = std::fs::File::create(path.clone()).unwrap();
        file.write_all(contents.as_bytes()).unwrap();
    }

    // output [tmp folder]

    let mut out = std::io::stdout();
    out.write_all(&format!("[{}]", folder.to_string_lossy()).as_bytes()).unwrap();
}

#[test]
fn can_generate_output() {
    let agrs = vec!["ignored".to_string(), "files".to_string(), "ABC=123".to_string(), "XYZ=987".to_string()];

    main_inner(agrs.into_iter());

    let tmp = std::env::temp_dir();
    println!("{:?}", tmp);
    let folder = tmp.join("env_subst");
    let mut files = std::fs::read_dir(folder).unwrap();

    let file = files.next().unwrap().unwrap();
    let path = file.path();
    let mut file = std::fs::File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents, "abc: 123\nxyz: 987");
}
