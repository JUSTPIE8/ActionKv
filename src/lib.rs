use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc::crc32;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, SeekFrom};
use std::path::Path;

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
            panic!(
                "data corruption encountered ({}!={}",
                checksum, saved_checksum
            );
        }

        let value = data.split_off(key_len as usize);
        let key = data;
        Ok(KeyValuePair { key, value })
    }
    pub fn insert(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<()> {
        let position = self.insert_but_ignore_index(key, value)?;
        self.index.insert(key.to_vec(), position);
        Ok(())
    }

    pub fn seek_to_end(&mut self) -> io::Result<u64> {
        self.f.seek(SeekFrom::End(0))
    }

    pub fn get(&mut self, key: &ByteStr) -> io::Result<Optionn<ByteString>> {
        let position = match self.index.get(key) {
            None => return Ok(None),
            Some(position) => *position,
        };
        let kv = self.get_at(position)?;
        Ok(Some(kv.value))
    }

    pub fn get_at(&mut self, position: u64) -> io::Result<KeyValuePair> {
        let mut f = BufReader::new(&mut self.f);
        f.seek(Seekfrom::Start(position))?;
        let kv = ActionKV::process_record(&mut f)?;
        Ok(kv)
    }

    pub fn find(&mut self, target: &ByteStr) -> io::Result<Option<(u64, ByteString)>> {}
    pub fn insert_but_ignore_index(&mut self, key: &ByteStr, value: &ByteStr) -> io::Result<u64> {
        Ok(5)
    }
}
