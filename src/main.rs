use std::{fs::{self, Metadata}, time::Duration, os::unix::prelude::MetadataExt, io::{stdin}};

const SEVEN_DAYS: Duration = Duration::new(60 * 60 * 24 * 7, 0);

#[derive(Debug)]
struct Entry {
	metadata: Metadata,
	path: String,
	size: u64
}

fn main() {
	let cleanup_paths: Vec<String>;
	if let Ok(result) = fs::read_to_string("cleanup-list.txt") {
		cleanup_paths = result.split('\n').map(|x| x.to_owned()).collect();
	} else {
		println!("Can't find required file \"cleanup-list.txt\"");
		return;
	}

	println!("Cleanup: {:?}", cleanup_paths);

	let files_directories = get_files_directories_list(&cleanup_paths);
	let files = files_directories.0;
	let directories = files_directories.1;

	println!("Files      : {}", files.len());
	println!("Directories: {}", directories.len());

	let considered_directories = get_considered_directories(&directories);
	println!("\nDirectories considered: {}", considered_directories.len());
	
	if considered_directories.len() == 0 {
		println!("Aborting...");
		return;
	}
	
	let freeable_size = get_freeable_size(&considered_directories);
	println!("\nSize to be freed: {:.2} {}", freeable_size.0, freeable_size.1);

	println!("Are you sure you want to delete these directories? (y/N): ");
	let mut input_buffer = String::new();
	if let Ok(_) = stdin().read_line(&mut input_buffer) {
		if input_buffer.trim() == "y" || input_buffer.trim() == "Y" {
			println!("Deleting...");

			for directory in &considered_directories {
				delete_directory(&directory.path);
			}

			println!("Deletion done.");
		} else {
			println!("Canceled.");
		}
	} else {
		println!("Canceled.");
	}
}

fn get_files_directories_list(cleanup_paths: &[String]) -> (Vec<Entry>, Vec<Entry>){
	let mut files = Vec::new();
	let mut directories = Vec::new();
	
	for cleanup_entry in cleanup_paths {
		if cleanup_entry.len() == 0 || cleanup_entry.starts_with("#") {
			continue;
		}

		if let Ok(files_dirs) = fs::read_dir(cleanup_entry) {
			for entry_result in files_dirs {
				let entry = entry_result.unwrap();
				let metadata = entry.metadata().unwrap();

				if metadata.is_dir() {
					let path = entry.path().to_string_lossy().to_string();
					directories.push(Entry {
						metadata: metadata,
						path: path.clone(),
						size: fs_extra::dir::get_size(path).unwrap()
					});
				} else {
					files.push(Entry {
						metadata: metadata.clone(),
						path: entry.path().to_string_lossy().to_string(),
						size: metadata.size()
					});
				}
			}
		} else {
			println!("Faied to find \"{}\"", cleanup_entry);
		}
	}
	(files, directories)
}

fn get_considered_directories(directories: &[Entry]) -> Vec<&Entry> {
	let mut considered_directories = Vec::new();
	for directory in directories {
		if directory.metadata.created().unwrap().elapsed().unwrap() > SEVEN_DAYS {
			considered_directories.push(directory);
		}
	}
	considered_directories
}

fn get_freeable_size(considered_directories: &[&Entry]) -> (f64, &'static str) {
	let mut considered_size = 0;
	for directory in considered_directories {
		considered_size += directory.size;
		let human_readable_size = get_human_readable_size(directory.size);
		println!("> {} {:.2} {}", directory.path, human_readable_size.0, human_readable_size.1);
	}

	get_human_readable_size(considered_size)
}

fn get_human_readable_size(size: u64) -> (f64, &'static str) {
	let sizes = ["B", "KB", "MB", "GB"];
	let mut size = size as f64;


	for i in 0..sizes.len() {
		if size >= 1024.0 {
			size /= 1024.0;
		} else {
			return (size, sizes[i]);
		}
	}

	(0.0, sizes[0])
}

fn delete_directory(path: &String) {
	println!("Deleting {}", path);
	match fs::remove_dir_all(path) {
		Err(err) => {
			println!("{}", err);
		},
		_ => {}
	}
}
