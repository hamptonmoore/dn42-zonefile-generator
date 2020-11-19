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

    zonefile.write_all(b"$ORIGIN .\n$TTL 300\ndn42. IN SOA once.i.get.a.tld.for.zone me.hampton.pw (1 7200 360 84600 300)\n")?;

    for path in paths {
        if let Ok(path) = path {
            if !path.path().to_str().unwrap().ends_with("dn42") {
                continue;
            }
            let file = File::open(path.path()).unwrap();
            let buf_reader = BufReader::new(file);
            let mut file_name: String = String::new();
            let mut name_server: String = String::new();

            for line in buf_reader.lines() {
                if let Ok(line) = line {
                    let line_split: Vec<String> = line.split_whitespace().map(|s| s.to_string()).collect();

                    match line_split[0].as_str() {
                        "domain:" => {
                            file_name = force_fqdn(line_split[1].clone());
                        }
                        "nserver:" => {
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
                        "ds-rdata:" => {
                            let (_start, dsrecord) = line.split_at(10);
                            add_record(&mut zonefile, file_name.clone(), "DS", dsrecord.trim().parse().unwrap())?;
                        }
                        _ => {}
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