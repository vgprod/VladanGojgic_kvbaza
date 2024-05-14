use libkvbaza::BazaKV;
use std::collections::HashMap;

#[cfg(target_os = "windows")]
const KORISCENJE: &str = "
KORISCENJE:
    kv_mem.exe FAJL uzmi KLJUC
    kv_mem.exe FAJL obrisi KLJUC
    kv_mem.exe FAJL unesi KLJUC VREDNOST
    kv_mem.exe FAJL azuriraj KLJUC VREDNOST
";

#[cfg(not(target_os = "windows"))]
const KORISCENJE: &str = "
KORISCENJE:
    kv_mem FAJL uzmi KLJUC
    kv_mem FAJL obrisi KLJUC
    kv_mem FAJL unesi KLJUC VREDNOST
    kv_mem FAJL azuriraj KLJUC VREDNOST
"; 

type BajtStr = [u8];
type BajtString = Vec<u8>;

fn sacuvaj_indeks_na_disku(a: &mut BazaKV, indeks_kljuc: &BajtStr) {
  a.indeks.remove(indeks_kljuc);
  let indeks_kao_bajt = bincode::serialize(&a.indeks).unwrap();
  a.indeks = std::collections::HashMap::new();
  a.unesi(indeks_kljuc, &indeks_kao_bajt).unwrap();
}

fn main() {
  const INDEKS_KLJUC: &BajtStr = b"+indeks";

  let argumenti: Vec<String> = std::env::args().collect();
  let ime_fajla = argumenti.get(1).expect(&KORISCENJE);
  let radnja = argumenti.get(2).expect(&KORISCENJE).as_ref();
  let kljuc = argumenti.get(3).expect(&KORISCENJE).as_ref();
  let mozda_vrednost = argumenti.get(4);

  let putanja = std::path::Path::new(&ime_fajla);
  let mut a = BazaKV::otvori(putanja).expect("neuspelo otvaranje fajla");

  a.ucitaj().expect("neuspelo ucitavanje podataka");

  match radnja {
    "uzmi" => {
      let indeks_kao_bajt = a.uzmi(&INDEKS_KLJUC)
                                    .unwrap()
                                    .unwrap();

      let indeks_dekodiran = bincode::deserialize(&indeks_kao_bajt);

      let indeks: HashMap<BajtString, u64> = indeks_dekodiran.unwrap();

      match indeks.get(kljuc) {
        None => eprintln!("{:?} nije nadjen", kljuc),
        Some(&i) => {
          let kv = a.uzmi_na(i).unwrap();
          println!("{:?}", kv.vrednost)             
        }
      }
    }

    "obrisi" => a.obrisi(kljuc).unwrap(),

    "unesi" => {
      let vrednost = mozda_vrednost.expect(&KORISCENJE).as_ref();
      a.unesi(kljuc, vrednost).unwrap();
      sacuvaj_indeks_na_disku(&mut a, INDEKS_KLJUC);     
    }

    "azuriraj" => {
      let vrednost = mozda_vrednost.expect(&KORISCENJE).as_ref();
      a.azuriraj(kljuc, vrednost).unwrap();
      sacuvaj_indeks_na_disku(&mut a, INDEKS_KLJUC);    
    }
    _ => eprintln!("{}", &KORISCENJE),
  }
}
