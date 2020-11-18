use std::{fs, io};
use std::fs::File;
use std::io::{BufReader, BufRead, BufWriter, Write};
use std::env;

fn main() -> io::Result<()>{

    let path = env::args().nth(1).expect("Please provide a zonefile directory as the first argument");
    let zonefile = env::args().nth(2).expect("Please provide a file for the zone file as the second argument");

    let paths = fs::read_dir(path).expect("Count not open zonefile directory");

    println!("Starting zonefile generation");

    let mut zonefile = BufWriter::new(File::create(zonefile).expect("Count not open output file for zonefile"));

    zonefile.write_all(b"$ORIGIN .\n$TTL 3600\n")?;

    for path in paths {
        if let Ok(path) = path {
            let file = File::open(path.path()).unwrap();
            let buf_reader = BufReader::new(file);
            let mut file_name: String = String::new();
            let mut name_server: String = String::new();

            for line in buf_reader.lines() {
                if let Ok(line) = line {
                    let line_split: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();
                    if line_split[0] == "domain:" {
                        file_name = force_fqdn(line_split[1].clone());
                    }
                    if line_split[0] == "nserver:" {
                        let ns: String = force_fqdn(line_split[1].clone());
                        if !ns.eq(&name_server){
                            add_record(&mut zonefile, file_name.clone(), "NS", ns.clone())?;
                            name_server = ns.clone();
                        }

                        if line_split.len() == 3 {
                            let ip: String = line_split[2].clone();
                            if ip.contains(":") {
                                add_record(&mut zonefile, ns.clone(), "AAAA", ip.clone())?;
                            } else {
                                add_record(&mut zonefile, ns.clone(), "A", ip.clone())?;
                            }
                        }
                    }
                }
            }
        }
    }

    zonefile.flush()?;

    Ok(())
}

fn add_record(buffer: &mut BufWriter<File>, domain: String, record_type: &str, data: String) -> io::Result<()>{
    buffer.write_all(domain.as_bytes())?;
    buffer.write_all(b"\tIN\t")?;
    buffer.write_all(record_type.as_bytes())?;
    buffer.write_all(b"\t")?;
    buffer.write_all(data.as_bytes())?;
    buffer.write_all(b"\n")?;

    Ok(())
}

fn force_fqdn(mut domain: String) -> String{
    if !domain.ends_with("."){
        domain.push_str(".");
    }

    domain
}