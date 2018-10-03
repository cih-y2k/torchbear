//! A basic example on how to use request fields from inside a lua script.
extern crate actix;
extern crate actix_lua;
extern crate actix_web;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate tera;
extern crate rlua;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate serde_urlencoded;
extern crate rlua_serde;
extern crate uuid;
extern crate comrak;
extern crate rust_sodium;
extern crate base64;
extern crate config;

use std::sync::Arc;
use actix::prelude::*;
use actix_lua::LuaActorBuilder;
use actix_web::{server as actix_server, App};
use tera::{Tera};
use rlua::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::exit;

mod lua_bindings;

mod app_state {
    pub struct AppState {
        pub lua: ::actix::Addr<::actix_lua::LuaActor>,
        pub tera: ::std::sync::Arc<::tera::Tera>,
    }
}

fn set_vm_globals(lua: &Lua, tera: Arc<Tera>, lib_path: &str, app_path: &str) -> Result<(), LuaError> {
    println!("Lua lib: {}", &lib_path);
    lua.exec::<()>(&format!(r#"
        package.path = package.path..";{}?.lua;{}?.lua"
        require "torchbear"
    "#, lib_path, app_path), None)?;

    //lua.exec::<()>(include_str!("managers/web_server.lua"), "web_server.lua").unwrap();

    lua_bindings::tera::init(lua, tera)?;
    lua_bindings::yaml::init(lua)?;
    lua_bindings::uuid::init(lua)?;
    lua_bindings::markdown::init(lua)?;
    lua_bindings::client::init(lua)?;
    lua_bindings::stringset::init(lua)?;

    Ok(())
}

pub fn start (path_str: &str) {
    let path = PathBuf::from(path_str);

    if !path.is_dir() {
        eprintln!("{} is not a directory", path_str);
        exit(1);
    }

    let mut settings_path = path.clone();
    settings_path.push("Settings.toml");

    let mut settings = config::Config::new();
    match settings.merge(config::File::with_name(settings_path.to_str().unwrap())) {
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
        Ok(_) => {}
    };
    settings.merge(config::Environment::with_prefix("torchbear"));

    let hashmap = settings.deserialize::<HashMap<String, String>>().unwrap();

    fn get_or (map: &HashMap<String, String>, key: &str, val: &str) -> String {
        map.get(key).map(|s| s.to_string()).unwrap_or(String::from(val))
    }

    let handler_path = get_or(&hashmap, "handler_path", "lua/handler.lua");
    let templates_path = get_or(&hashmap, "templates_path", "templates/**/*");
    let host = get_or(&hashmap, "host", "0.0.0.0:3000");

    let mut app_path = path.clone();
    app_path.push(get_or(&hashmap, "application", "./"));

    let mut lib_path = path.clone();
    lib_path.push(get_or(&hashmap, "torchbear_ext", "torchbear-ext/"));

    let sys = actix::System::new("actix-lua-web");
    let tera = Arc::new(compile_templates!(&templates_path));

    let shared_tera = tera.clone();
    let addr = Arbiter::start(move |_| {
        let tera = shared_tera;
        let lua_actor = LuaActorBuilder::new()
            //.on_handle(&handler_path)
            .on_handle_with_lua(include_str!("managers/web_server.lua"))
            .with_vm(move |vm| {
                set_vm_globals(vm, tera.clone(), lib_path.to_str().unwrap(), app_path.to_str().unwrap())
            })
            .build()
            .unwrap();

        lua_actor
    });

    actix_server::new(move || {
        App::with_state(app_state::AppState { lua: addr.clone(), tera: tera.clone() })
            .default_resource(|r| r.with(lua_bindings::server::handler))
    }).bind(&host)
        .unwrap()
        .start();

    println!("Started http server: localhost:3000");
    let _ = sys.run();
}
