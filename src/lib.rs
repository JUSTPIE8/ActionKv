use byteorder::LittleEndian;
use serde::{Deserialize, Serialize};

//since u8 represents a string internally
type ByteString = Vec<u8>;

type ByteStr = [u8];

#[derive(Debug, Serialize, Deserialize)]
pub struct KeyValuePair {
    pub key: ByteString,
    pub value: ByteString,
}

#[derive(Debug)]
pub struct ActionKV {
    f: File,
    pub index: HashMap<ByteString, u64>,
}
impl ActionKV {
    pub fn open(path: &Path) -> io::Result<Self> {
        let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .append(true)
            .open(path)?;
        let index = HashMap::new();
        Ok(ActionKV { f, index })
    }

    pub fn load(&mut self) -> io::Result<()> {
        let mut f = BufReader::new(&mut self.f);
        loop {
            let position = f.seek(SeekFrom::Current(0))?;
            let maybe_kv = ActionKV::process_record(&mut f);
            let kv = match maybe_kv {
                Ok(kv) => kv,
                Err(err) => match err.kind() {
                    io::ErrorKind::UnexpectedEof => {
                        break;
                    }
                    _ => return Err(err),
                },
            };
            self.index.insert(kv.key, position);
        }
        Ok(())
    }
}

fn process_record<R: Read>(f: &mut R) -> io::Result<KeyValuePair> {
    let saved_checksum = f.read_u32::<LittleEndian>()?;

    let key_len = f.read_u32::<LittleEndian>()?;
    let val_len = f.read_u32::<LittleEndian>()?;
    let data_len = key_len + val_len;

    let mut data = ByteString::with_capacity(data_len as usize);

    {
        f.by_ref().take(data_len as u64).read_to_end(&mut data)?;
    }

    debug_assert_eq!(data.len(), data_len as usize);
    let checksum = crc32::checksum_ieee(&data);
    if checksum != saved_checksum {
        panic!("data corruption encountered ({}!={}"),checksum,saved_checksum);
    }

    let value = data.split_off(key_len as usize);
    let key = data;
    Ok(KeyValuePair { key, value })
}
