#![windows_subsystem = "windows"]

extern crate native_windows_gui as nwg;

use nwg::NativeUi;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use regex::Regex;

fn get_modloader_path() -> PathBuf {
    #[cfg(debug_assertions)]
    let mut path = PathBuf::from(r"d:\Repositories\modloader-profile-switcher\reference");
    #[cfg(not(debug_assertions))]
    let mut path = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    path.push("modloader");
    path.push("modloader.ini");
    path
}

fn parse_ini(path: &PathBuf) -> Option<(String, Vec<String>)> {
    let content = fs::read_to_string(path).ok()?;
    
    // Find current profile
    let re_current = Regex::new(r"(?i)^\s*Profile\s*=\s*(.*?)\s*(?:;|$)").unwrap();
    let mut current_profile = String::new();
    for line in content.lines() {
        if let Some(caps) = re_current.captures(line) {
            current_profile = caps[1].trim().to_string();
            break;
        }
    }

    // Find all profiles
    let re_profile = Regex::new(r"(?i)^\s*\[Profiles\.([^.]+)\.").unwrap();
    let mut profiles = Vec::new();
    for line in content.lines() {
        if let Some(caps) = re_profile.captures(line) {
            let profile_name = caps[1].trim().to_string();
            if !profiles.contains(&profile_name) {
                profiles.push(profile_name);
            }
        }
    }

    Some((current_profile, profiles))
}

fn update_ini(path: &PathBuf, new_profile: &str) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let mut new_content = String::new();
    let re_current = Regex::new(r"(?i)^(\s*Profile\s*=)\s*(.*?)\s*(;.*)?$").unwrap();
    
    let mut updated = false;
    for line in content.lines() {
        if !updated && re_current.is_match(line) {
            let replaced = re_current.replace(line, |caps: &regex::Captures| {
                let prefix = &caps[1];
                let comment = caps.get(3).map_or("", |m| m.as_str());
                if comment.is_empty() {
                    format!("{} {}", prefix, new_profile)
                } else {
                    let old_profile = &caps[2];
                    let mut padding = String::new();
                    if new_profile.len() < old_profile.len() {
                        padding = " ".repeat(old_profile.len() - new_profile.len());
                    }
                    format!("{} {}{} {}", prefix, new_profile, padding, comment)
                }
            });
            new_content.push_str(&replaced);
            updated = true;
        } else {
            new_content.push_str(line);
        }
        new_content.push_str("\r\n");
    }

    fs::write(path, new_content).is_ok()
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    let ini_path = get_modloader_path();
    let (current_profile, profiles) = match parse_ini(&ini_path) {
        Some(res) => res,
        None => {
            nwg::error_message("Error", &format!("Could not read modloader.ini at {}", ini_path.display()));
            return;
        }
    };

    if profiles.is_empty() {
        nwg::error_message("Error", "No profiles found in modloader.ini");
        return;
    }

    let mut icon = Default::default();
    let _ = nwg::Icon::builder()
        .source_bin(Some(include_bytes!("../icon.ico")))
        .build(&mut icon);

    let mut window = Default::default();
    nwg::Window::builder()
        .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE | nwg::WindowFlags::MINIMIZE_BOX)
        .size((350, 20 + (profiles.len() as i32) * 40))
        .position((300, 300))
        .title("Modloader Profile Switcher")
        .icon(Some(&icon))
        .build(&mut window)
        .unwrap();

    let mut layout = Default::default();
    nwg::GridLayout::builder()
        .parent(&window)
        .margin([10, 10, 10, 10])
        .spacing(5)
        .build(&mut layout)
        .unwrap();

    let mut buttons = Vec::new();

    for (i, p) in profiles.iter().enumerate() {
        let mut btn = Default::default();
        let is_current = p == &current_profile;
        let text = if is_current { format!("{} (Active)", p) } else { p.clone() };
        
        nwg::Button::builder()
            .text(&text)
            .enabled(!is_current)
            .parent(&window)
            .build(&mut btn)
            .unwrap();

        layout.add_child(0, i as u32, &btn);
        buttons.push(btn);
    }

    let window_handle = window.handle;
    let buttons_rc = Rc::new(buttons);
    let ini_path_rc = Rc::new(ini_path);
    let profiles_rc = Rc::new(profiles);

    let handler = nwg::full_bind_event_handler(&window_handle, move |evt, _evt_data, handle| {
        use nwg::Event as E;

        match evt {
            E::OnWindowClose => {
                if handle == window_handle {
                    nwg::stop_thread_dispatch();
                }
            },
            E::OnButtonClick => {
                for (i, btn) in buttons_rc.iter().enumerate() {
                    if handle == btn.handle {
                        let new_profile = &profiles_rc[i];
                        if update_ini(&ini_path_rc, new_profile) {
                            nwg::simple_message("Success", &format!("Profile changed to {}", new_profile));
                            nwg::stop_thread_dispatch();
                        } else {
                            nwg::error_message("Error", "Failed to update modloader.ini");
                        }
                    }
                }
            },
            _ => {}
        }
    });

    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);
}