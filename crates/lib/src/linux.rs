use std::{env, path::PathBuf, process::Command};

use nix::unistd::{Group, User};

/// Get the home folder of the user
/// Panic if no home folder is set (is it really possible ?)
pub fn get_home() -> PathBuf {
    let home = env::home_dir();
    if home.is_none() {
        panic!("Could not find your home directory.")
    } else {
        home.unwrap()
    }
}

pub fn upsert_group(group: &str) -> Result<Group, anyhow::Error> {
    // check if the group exists
    if let Some(g) = Group::from_name(&group)? {
        return Ok(g);
    }

    let group_success = Command::new("groupadd").arg(group).status()?;

    if !group_success.success() {
        let error_message = format!("error while creating the group: {:?}", group_success.code());
        return Err(anyhow::Error::msg(error_message));
    }

    let g = Group::from_name(&group)?.unwrap();
    Ok(g)
}

pub fn add_user_to_group(user: User, group: &Group) -> Result<(), anyhow::Error> {
    let command_success = Command::new("adduser")
        .args(vec![user.name, (&group.name).clone()])
        .status()?;

    if !command_success.success() {
        let error_message = format!(
            "error while adding user to the group: {:?}",
            command_success.code()
        );
        return Err(anyhow::Error::msg(error_message));
    }

    Ok(())
}

pub fn get_user() -> User {
    let name =
        String::from_utf8_lossy(&Command::new("logname").output().unwrap().stdout).to_string();
    let user = User::from_name(name.trim()).unwrap();

    user.unwrap()
}
