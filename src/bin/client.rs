use std::error;
use rrss_lib::bug::Bug;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn main() -> Result<()> {
    match start() {
        Ok(_) => (),
        Err(e) => handle_error(e)?
    }

    Ok(())
}

fn start() -> Result<()> {
    #[cfg(not(debug_assertions))]
    {
        use auto_launch::*;
        use std::env;

        let auto = AutoLaunchBuilder::new()
            .set_app_name("rrss")
            .set_app_path(env::current_exe()?.to_str().unwrap())
            .set_use_launch_agent(true)
            .build()?;
        if !auto.is_enabled()? {
            auto.enable()?;
        }
    }

    Ok(())
}

fn handle_error(e: Box<dyn error::Error>) -> Result<()> {
    let bug = Bug {
        body: e.to_string(),
        machine: gethostname::gethostname().to_str().unwrap().to_string()
    };

    let client = reqwest::blocking::Client::new();
    client.post("http://50.116.44.160:9000/bug")
            .body(serde_json::to_string(&bug).unwrap())
        .send()?;

    Ok(())
}