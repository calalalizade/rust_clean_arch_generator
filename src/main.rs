#![allow(warnings)]

use handlebars::Handlebars;
use std::{ env, fs, path::Path };
use serde_json::json;
use std::collections::HashMap;

fn main() {
    // 1. Input Handling
    let feature_name = std::env::args().nth(1).expect("Feature name argument required");

    // 2. Template Registration
    let config: toml::Value = toml
        ::from_str(
            &fs
                ::read_to_string("template_config.toml")
                .expect("Unable to read template configuration file")
        )
        .expect("Invalid configuration format");

    let mut handlebars = Handlebars::new();
    if let Some(templates) = config.get("templates").and_then(toml::Value::as_table) {
        for (name, path) in templates {
            handlebars
                .register_template_file(
                    name,
                    path.as_str().expect("Template path should be a string")
                )
                .expect(&format!("Can't find template {}", name));
        }
    }

    // 3. Data Preparation
    let data =
        json!({
        "feature_name": feature_name,
        "snake_case_feature_name": feature_name.to_lowercase().replace("-", "_"),
        "capitalize_feature_name": feature_name.split('_')
                                                .map(|word| word.chars().
                                                next().unwrap().to_uppercase().collect::<String>() + &word[1..]).
                                                collect::<Vec<String>>().join(""),
    });

    // 4. Create feature folder and subfolders
    let feature_dir = format!("src/features/{}", feature_name.to_lowercase().replace("-", "_"));
    fs::create_dir_all(&feature_dir).unwrap();
    fs::write(format!("{}/mod.rs", feature_dir), "").unwrap(); // Create mod.rs

    for subfolder in ["domain", "application", "infrastructure", "interface"] {
        let subfolder_path = format!("{}/{}", feature_dir, subfolder);
        fs::create_dir_all(&subfolder_path).unwrap();
        fs::write(format!("{}/mod.rs", subfolder_path), "").unwrap();

        let layer_subfolders: HashMap<&str, Vec<&str>> = HashMap::from([
            ("application", vec!["di", "interactor", "use_case", "model", "util"]),
            ("domain", vec!["entity", "repository", "interactor"]),
            (
                "infrastructure",
                vec!["data_access", "dto", "enum", "mapper", "repository", "mapper"],
            ),
            ("interface", vec!["controller", "model"]),
        ]);

        if let Some(subfolders) = layer_subfolders.get(subfolder) {
            for sub in subfolders {
                let dir_path = format!("{}/{}", subfolder_path, sub);
                let file_path = format!("{}/mod.rs", dir_path);
                fs::create_dir_all(&dir_path).unwrap(); // Create directory (if it doesn't exist)
                fs::write(file_path, "").unwrap(); // Create the mod.rs file (empty)
            }
        }
    }

    // Feature-level mod.rs
    let mod_rs_content =
        "pub mod application;\npub mod domain;\npub mod infrastructure;\npub mod interface;";
    fs::write(format!("{}/mod.rs", feature_dir), mod_rs_content).unwrap();

    // Application layer
    let app_subfolders = &[
        ("di", "pub mod container;".to_string()),
        ("interactor", format!("pub mod i_{}_interactor;", feature_name)),
        ("use_case", format!("pub mod {}_use_case;", feature_name)),
        ("model", "".to_string()),
        ("util", "".to_string()),
    ];
    create_layer_structure(&feature_dir, "application", app_subfolders);

    // Domain layer
    let domain_subfolders = &[
        ("interactor", format!("pub mod {}_interactor_impl;", feature_name)),
        ("repository", format!("pub mod i_{}_repository;", feature_name)),
        ("entity", "".to_string()),
    ];
    create_layer_structure(&feature_dir, "domain", domain_subfolders);

    // Infrastructure layer
    let infra_subfolders = &[
        ("data_access", format!("pub mod {}_data_source;", feature_name)),
        ("repository", format!("pub mod {}_repository_impl;", feature_name)),
        ("dto", "".to_string()),
        ("enum", "".to_string()),
        ("mapper", "".to_string()),
    ];
    create_layer_structure(&feature_dir, "infrastructure", infra_subfolders);

    // Interface layer
    let interface_subfolders = &[
        ("controller", format!("pub mod {}_controller;", feature_name)),
        ("model", "".to_string()),
    ];
    create_layer_structure(&feature_dir, "interface", interface_subfolders);

    // 5. Template Rendering
    // Application Layer
    let output_container = handlebars.render("container", &data).expect("Render error");
    let output_interactor = handlebars.render("i_interactor", &data).expect("Render error");
    let output_use_case = handlebars.render("use_case", &data).expect("Render error");

    // Domain Layer
    let output_repository = handlebars.render("i_repository", &data).expect("Render error");
    let output_interactor_impl = handlebars.render("interactor_impl", &data).expect("Render error");

    // Infrastructure Layer
    let output_data_source = handlebars.render("data_source", &data).expect("Render error");
    let output_repository_impl = handlebars.render("repository_impl", &data).expect("Render error");

    // Interface Layer
    let output_controller = handlebars.render("controller", &data).expect("Render error");

    // 6. File Output (adjusted to include subfolders)
    // Application layer
    fs::write(format!("{}/application/di/container.rs", feature_dir), output_container).unwrap();
    fs::write(
        format!("{}/application/interactor/i_{}_interactor.rs", feature_dir, feature_name),
        output_interactor
    ).unwrap();
    fs::write(
        format!("{}/application/use_case/{}_use_case.rs", feature_dir, feature_name),
        output_use_case
    ).unwrap();

    // Domain layer
    fs::write(
        format!("{}/domain/repository/i_{}_repository.rs", feature_dir, feature_name),
        output_repository
    ).unwrap();
    fs::write(
        format!("{}/domain/interactor/{}_interactor_impl.rs", feature_dir, feature_name),
        output_interactor_impl
    ).unwrap();

    // Infrastructure layer
    fs::write(
        format!("{}/infrastructure/data_access/{}_data_source.rs", feature_dir, feature_name),
        output_data_source
    ).unwrap();
    fs::write(
        format!("{}/infrastructure/repository/{}_repository_impl.rs", feature_dir, feature_name),
        output_repository_impl
    ).unwrap();

    // Interface layer
    fs::write(
        format!("{}/interface/controller/{}_controller.rs", feature_dir, feature_name),
        output_controller
    ).unwrap();

    println!("Generated container.rs for feature: {}", feature_name);
}

// Helper function to create the structure of a layer
fn create_layer_structure(base_dir: &str, layer_name: &str, subfolders: &[(&str, String)]) {
    // Create the layer's main mod.rs
    let layer_mod_rs_content = subfolders
        .iter()
        .map(|(subfolder, _)| format!("pub mod {};", subfolder))
        .collect::<Vec<String>>()
        .join("\n");

    fs::write(format!("{}/{}/mod.rs", base_dir, layer_name), layer_mod_rs_content).unwrap();

    // Create mod.rs files within each subfolder
    for (subfolder, content) in subfolders {
        let path = format!("{}/{}/{}/mod.rs", base_dir, layer_name, subfolder);
        fs::write(path, content).unwrap();
    }
}
