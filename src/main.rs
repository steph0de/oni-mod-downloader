use std::fs::{File, remove_dir_all};
use std::io::{self, BufRead, Write};
use std::path::Path;
use log::{info, warn};
use steam_workshop_api::{Workshop, WorkshopItem};
use unzip::Unzipper;

fn lines_from_text_file(filename: impl AsRef<Path>) -> Vec<String> {
    let file = File::open(filename).expect("no such file");
    let buf = io::BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not get line").parse().expect("Could not parse data"))
        .collect::<Vec<String>>()
}

fn sanitize_string(txt: String) -> String {
    txt.replace("/","")
        .replace(|c: char| !c.is_ascii(), "")
        .replace("  ", " ")
        .trim().to_string()
}

fn download_file(url: String, dest: &String) -> Result<(), Box<dyn std::error::Error>>{
    let resp = reqwest::blocking::get(url)?
        .bytes().expect("Error fetching file");
    let mut f = File::create(dest)?;
    
    f.write_all(&resp)?;
    f.flush()?; 
    Ok(())
}

fn install_mod(filepath: &String, dest: String){

    let zip_file = File::open(filepath).expect("Could not open zip file");
    match Unzipper::new(zip_file, &dest).unzip(){
        Ok(_)=> info!("Mod installed successfully"),
        Err(err) => {
            panic!("{}", err);
        }
    };
    
}

fn clean_mod_directory(dir: String) -> std::io::Result<()> {
    std::fs::remove_dir_all(format!("mods/{}/archived_versions", dir))?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workshop = Workshop::new(None);
    let _ids: Vec<String> = lines_from_text_file("./modlist.txt");
    let details: Vec<WorkshopItem> = match workshop.get_published_file_details(&_ids) {
        Ok(details) => details,
        Err(err) => {
            panic!("Failed to get file info {}", err);
        }
    };
    for x in details{
        let title = sanitize_string(x.title);
        let archive_filename = format!("mods/{}.zip", title).to_string();
        let outdir = format!("mods/{}", title);

        info!("{:?}", title);
        info!("{:?}", x.file_url);

        info!("Downloading mod: {}", title);
        download_file(x.file_url, &archive_filename)?;
        install_mod(&archive_filename, outdir);
        clean_mod_directory(title)?;
    }
    Ok(())
}
