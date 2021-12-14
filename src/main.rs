use std::env;
use std::process;
use std::process::Command;
use std::os::unix::process::CommandExt;

fn get_env_var(env_var: &str) -> String {
    env::var(env_var).unwrap_or_else(|error| {
        println!("[vault2env] {} variable missing: {:?}", env_var, error);
        println!("[vault2env] Running application anyway and hoping for the best 🤞");
        clean_env_vars();
        exec_app();
        process::exit(1);
    })
}

fn clean_env_vars() {
    for key in ["VAULT_TOKEN", "VAULT_SERVER", "VAULT_PATH"] {
        env::remove_var(key);
    }
}

fn exec_app() {
    println!("[vault2env] Executing: {:?}", env::args().skip(1).collect::<Vec<String>>());
    Command::new("env")
        .args(env::args_os().skip(1))
        .exec();
}

struct Config {
    token:  String,
    server: String,
    path:   String,
}

impl Config {
    fn set() -> Config {
        Config {
            token:  get_env_var("VAULT_TOKEN"),
            server: get_env_var("VAULT_SERVER"),
            path:   get_env_var("VAULT_PATH"),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let vault_conf = Config::set();
    clean_env_vars();

    let url = format!("{}/v1/{}", vault_conf.server, vault_conf.path);

    let object: serde_json::Value = ureq::get(&url)
        .set("X-Vault-Token", &vault_conf.token)
        .call()?
        .into_json()?;

    for (key, value) in object["data"]["data"].as_object().unwrap() {
        println!("[vault2env] Setting env var: {}", key);
        env::set_var(key, value.as_str().unwrap());
    }

    exec_app();
    Ok(())
}
