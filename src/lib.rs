use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, BufWriter, SeekFrom};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use crc::crc32;
use serde_derive::{Deserialize, Serialize};

type ByteString = Vec<u8>;
type ByteStr = [u8];

#[derive(Debug, Serialize, Deserialize)]
pub struct KljucVrednostPar {
  pub kljuc: ByteString,
  pub vrednost: ByteString,
}

#[derive(Debug)] 
pub struct BazaKV {
  f: File,
  pub indeks: HashMap<ByteString, u64>,
}

impl BazaKV {
    pub fn otvori(path: &Path) -> io::Result<Self> {
      let f = OpenOptions::new() 
        .read(true) 
        .write(true)
        .create(true)
        .append(true)
        .open(path)?;
      let indeks = HashMap::new();
      Ok(BazaKV { f, indeks })
    }
  
    fn obradi_zapis<R: Read>(f: &mut R) -> io::Result<KljucVrednostPar> {
      let sac_kontr_zbir = f.read_u32::<LittleEndian>()?;
      let kljuc_duz = f.read_u32::<LittleEndian>()?;
      let vred_duz = f.read_u32::<LittleEndian>()?;
      let podatak_duz = kljuc_duz + vred_duz;
  
      let mut podatak = ByteString::with_capacity(podatak_duz as usize);
  
      {
        f.by_ref().take(podatak_duz as u64).read_to_end(&mut podatak)?;
      }
      debug_assert_eq!(podatak.len(), podatak_duz as usize);
  
      let kontrolni_zbir = crc32::checksum_ieee(&podatak);
      if kontrolni_zbir != sac_kontr_zbir {
        panic!(
          "pronadjeni osteceni podaci ({:08x} != {:08x})",
          kontrolni_zbir, sac_kontr_zbir
        );
      }
  
      let vrednost = podatak.split_off(kljuc_duz as usize);
      let kljuc = podatak;
  
      Ok(KljucVrednostPar { kljuc, vrednost })
    }
  
    pub fn idi_na_kraj(&mut self) -> io::Result<u64> {
      self.f.seek(SeekFrom::End(0))
    }
  
    pub fn ucitaj(&mut self) -> io::Result<()> {
      let mut f = BufReader::new(&mut self.f);
  
      loop {
        let trenutna_pozicija = f.seek(SeekFrom::Current(0))?;
  
        let mozda_kv = BazaKV::obradi_zapis(&mut f);
        let kv = match mozda_kv {
          Ok(kv) => kv,
          Err(greska) => {
            match greska.kind() {
              io::ErrorKind::UnexpectedEof => {
                break;
              }
              _ => return Err(greska),
            }
          }
        };
  
        self.indeks.insert(kv.kljuc, trenutna_pozicija);
      }
  
      Ok(())
    }
  
    pub fn uzmi(&mut self, kljuc: &ByteStr) -> io::Result<Option<ByteString>> {
      let pozicija = match self.indeks.get(kljuc) {
        None => return Ok(None),
        Some(pozicija) => *pozicija,
      };
  
      let kv = self.uzmi_na(pozicija)?;
  
      Ok(Some(kv.vrednost))
    }
  
    pub fn uzmi_na(&mut self, pozicija: u64) -> io::Result<KljucVrednostPar> {
      let mut f = BufReader::new(&mut self.f);
      f.seek(SeekFrom::Start(pozicija))?;
      let kv = BazaKV::obradi_zapis(&mut f)?;
  
      Ok(kv)
    }
  
    pub fn nadji(
      &mut self,
      trazen: &ByteStr,
    ) -> io::Result<Option<(u64, ByteString)>> {
      let mut f = BufReader::new(&mut self.f);
  
      let mut nadjen: Option<(u64, ByteString)> = None;
  
      loop {
        let pozicija = f.seek(SeekFrom::Current(0))?;
  
        let mozda_kv = BazaKV::obradi_zapis(&mut f);
        let kv = match mozda_kv {
          Ok(kv) => kv,
          Err(greska) => {
            match greska.kind() {
              io::ErrorKind::UnexpectedEof => {
                break;
              }
              _ => return Err(greska),
            }
          }
        };
  
        if kv.kljuc == trazen {
          nadjen = Some((pozicija, kv.vrednost));
        }
  
      }
  
      Ok(nadjen)
    }
  
    pub fn unesi(
      &mut self,
      kljuc: &ByteStr,
      vrednost: &ByteStr,
    ) -> io::Result<()> {
      let pozicija = self.unesi_ali_zanemari_indeks(kljuc, vrednost)?;
  
      self.indeks.insert(kljuc.to_vec(), pozicija);
      Ok(())
    }
  
    pub fn unesi_ali_zanemari_indeks(
      &mut self,
      kljuc: &ByteStr,
      vrednost: &ByteStr,
    ) -> io::Result<u64> {
      let mut f = BufWriter::new(&mut self.f);
  
      let kljuc_duz = kljuc.len();
      let vred_duz = vrednost.len();
      let mut tmp = ByteString::with_capacity(kljuc_duz + vred_duz);
  
      for bajt in kljuc {
        tmp.push(*bajt);
      }
  
      for bajt in vrednost {
        tmp.push(*bajt);
      }
  
      let kontrolni_zbir = crc32::checksum_ieee(&tmp);
  
      let sledeci_bajt = SeekFrom::End(0);
      let trenutna_pozicija = f.seek(SeekFrom::Current(0))?;
      f.seek(sledeci_bajt)?;
      f.write_u32::<LittleEndian>(kontrolni_zbir)?;
      f.write_u32::<LittleEndian>(kljuc_duz as u32)?;
      f.write_u32::<LittleEndian>(vred_duz as u32)?;
      f.write_all(&tmp)?;
  
      Ok(trenutna_pozicija)
    }
  
    #[inline]
    pub fn azuriraj(
      &mut self,
      kljuc: &ByteStr,
      vrednost: &ByteStr,
    ) -> io::Result<()> {
      self.unesi(kljuc, vrednost)
    }
  
    #[inline]
    pub fn obrisi(&mut self, kljuc: &ByteStr) -> io::Result<()> {
      self.unesi(kljuc, b"")
    }
  }