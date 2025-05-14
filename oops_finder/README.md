# oops_finder

`oops_finder` is a humorous file search CLI tool written in Rust. It allows users to search for files in a directory based on a pattern, with support for recursive search.

## Features
- Search for files by name pattern.
- Specify the directory to search in (defaults to the current directory).
- Recursive search through subdirectories.

## Usage

### Installation
1. Clone the repository:
   ```
   git clone https://github.com/NoEdgeAtLife/random_projects.git
   ```
2. Navigate to the `oops_finder` directory:
   ```
   cd random_projects/oops_finder
   ```
3. Build the project:
   ```
   cargo build --release
   ```

### Running the Tool
Run the tool with the following command:
```
./target/release/oops_finder -p <PATTERN> [-d <DIRECTORY>]
```

- `-p, --pattern`: The pattern to search for (required).
- `-d, --directory`: The directory to search in (optional, defaults to the current directory).

### Example
Search for files containing "example" in the current directory:
```
./target/release/oops_finder -p example
```

Search for files containing "test" in the `/home/user/documents` directory:
```
./target/release/oops_finder -p test -d /home/user/documents
```

## License
This project is licensed under the MIT License. See the [LICENSE](../LICENSE) file for details.
