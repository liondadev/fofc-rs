use std::error::Error;
use std::io::{Cursor, Read, Write};
use std::time::{SystemTime, UNIX_EPOCH};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

#[derive(Debug)]
pub struct Container {
    pub comment: String,
    pub x: u64,
    pub y: u64,
    pub z: u64,
    pub files: Vec<File>
}

#[derive(Clone, Debug)]
pub struct File {
    pub name: String,
    pub content: Vec<u8>
}

pub const Y_DIFFERENCE: u64 = 43;
pub const Z_DIFFERENCE: u64 = 34;
pub const MAGIC_NUMBER: u8 = 0x46;

fn read_string_until_0x00(cursor: &mut Cursor<&[u8]>) -> Result<String, Box<dyn Error>> {
    let mut buffer: Vec<u8> = Vec::new();

    loop {
        let mut byte = [0; 1];
        cursor.read_exact(&mut byte)?;
        if byte[0] == 0x00 {
            break;
        }

        buffer.push(byte[0])
    }

    let string = String::from_utf8_lossy(&buffer).into_owned();
    Ok(string)
}

impl Container {
    pub fn new(comment: &str) -> Result<Container, Box<dyn Error>> {
        let x = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        return Ok(Container {
            comment: comment.to_string(),
            x,
            y: x + Y_DIFFERENCE,
            z: x + Z_DIFFERENCE,
            files: vec![]
        })
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Container, Box<dyn Error>> {
        let mut cursor = Cursor::new(bytes);

        if cursor.read_u8()? != MAGIC_NUMBER {
            return Err(Box::from("invalid or incorrect magic number"));
        }

        let comment = read_string_until_0x00(&mut cursor)?;
        let x = cursor.read_u64::<LittleEndian>()?;
        let y = x + Y_DIFFERENCE;
        let z  = x + Z_DIFFERENCE;
        let file_count = cursor.read_u16::<LittleEndian>()?;

        let mut files: Vec<File> = Vec::new();

        for _ in 1..=file_count {
            let name = read_string_until_0x00(&mut cursor)?;
            let length = cursor.read_u64::<LittleEndian>()?;
            let mut content = vec![0; length as usize];
            cursor.read_exact(&mut content)?;
            files.push(File {
                name,
                content
            })
        }


        Ok(Container {x, y, z, comment, files})
    }

    pub fn add_file(&mut self, file: File) {
        self.files.push(file)
    }

    pub fn remove_file(&mut self, name: String) {
        self.files = self.files.iter().cloned().filter(|f| f.name != name).collect()
    }

    pub fn get_file(&self, name: String) -> Option<&File> {
        self.files.iter().find(|f| f.name == name)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.push(MAGIC_NUMBER);
        bytes.write(self.comment.as_bytes())?;
        bytes.push(0x00);
        bytes.write_u64::<LittleEndian>(self.x)?;
        bytes.write_u16::<LittleEndian>(self.files.len() as u16)?;

        for f in self.files.iter() {
            bytes.write(f.name.as_bytes())?;
            bytes.push(0x00);
            bytes.write_u64::<LittleEndian>(f.content.len() as u64)?;
            bytes.write_all(f.content.as_slice())?;
        }

        Ok(bytes)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_container_has_correct_values() {
        let mut container = Container::new("Example").unwrap();
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        assert_eq!(container.x, current_time);
        assert_eq!(container.y, current_time + Y_DIFFERENCE);
        assert_eq!(container.z, current_time + Z_DIFFERENCE);
        assert_eq!(container.files.len(), 0); // files are empty

        // ensure you can add files
        let file_name = "C:\\farting.png".to_string();
        let file = File {
            name: file_name.clone(),
            content: vec![0x00, 0xF2]
        };
        container.add_file(file);
        assert_eq!(container.files.len(), 1);

        container.remove_file(file_name);
        assert_eq!(container.files.len(), 0);
    }

    #[test]
    fn read_write() {
        let mut container = Container::new("The Best In The World").unwrap();

        let file_name = "C:\\hello.png".to_string();
        let file_content: [u8; 4] = [0x66, 0x66, 0x66, 0x66];
        let file = File {name: file_name, content: file_content.to_vec()};
        container.add_file(file);
        let file2 = File {name: "better file name!!!!".to_string(), content: [0x23, 0x54, 0xFF].to_vec()};
        container.add_file(file2);

        let as_bytes = container.to_bytes().unwrap();

        let new_container = Container::from_bytes(as_bytes.as_slice()).unwrap();

        println!("{:?}", new_container.files);
    }
}
